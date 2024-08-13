use crate::*;
use bevy::app::AppExit;
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use wgpu::*;

pub struct RenderPassContainer {
    pub texture: SurfaceTexture,
    pub render_pass: RenderPass<'static>,
    pub view: TextureView,
    pub encoder: CommandEncoder,
}
#[derive(Resource)]
pub struct Renderer {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub adapter: Adapter,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,
    pub render_pass: Option<RenderPassContainer>,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
}
impl Renderer {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 && height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture = Self::create_depth_texture(&self.device, &self.config);
        self.depth_view = self.depth_texture.create_view(&Default::default());

        info!("Surface resized to {}x{}", width, height);
    }
    fn create_depth_texture(device: &Device, config: &SurfaceConfiguration) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("Depth texture"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }
}
impl FromWorld for Renderer {
    fn from_world(world: &mut World) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: if cfg!(debug_assertions) {
                InstanceFlags::debugging()
            } else {
                InstanceFlags::empty()
            },
            dx12_shader_compiler: Dx12Compiler::default(),
            gles_minor_version: Gles3MinorVersion::default(),
        });

        let mut handle_q =
            world.query_filtered::<(&RawHandleWrapper, &Window), With<PrimaryWindow>>();
        let (raw_handle, window) = handle_q.single_mut(world);

        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: raw_handle.display_handle,
                raw_window_handle: raw_handle.window_handle,
            })
        }
        .unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("No suitable adapters found.");

        let config = surface
            .get_default_config(&adapter, window.physical_width(), window.physical_height())
            .expect("The surface is not supported by this adapter.");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Request device"),
                required_features: Features::SPIRV_SHADER_PASSTHROUGH,
                required_limits: Limits::downlevel_defaults(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .unwrap();

        surface.configure(&device, &config);

        let depth_texture = Self::create_depth_texture(&device, &config);
        let depth_view = depth_texture.create_view(&Default::default());

        Self {
            instance,
            surface,
            adapter,
            config,
            device,
            queue,
            render_pass: None,
            depth_texture,
            depth_view,
        }
    }
}
#[derive(Debug, Clone, Copy, Resource)]
pub struct ClearColor(pub Color);
#[derive(Debug, Event)]
struct SurfaceErrorEvent(SurfaceError);

fn handle_surface_error(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut renderer: ResMut<Renderer>,
    mut app_exit_event: ResMut<Events<AppExit>>,
    mut error_event: EventReader<SurfaceErrorEvent>,
) {
    for event in error_event.read() {
        match &event.0 {
            SurfaceError::Outdated | SurfaceError::Lost => {
                let window = window_q.single();
                renderer.resize(window.physical_width(), window.physical_height());
            }
            SurfaceError::OutOfMemory => {
                error!("Out of memory!");
                app_exit_event.send(AppExit);
                return;
            }
            e => error!("Surface error: {}", e),
        }
    }
}
fn on_resize(mut renderer: ResMut<Renderer>, window_q: Query<&Window, With<PrimaryWindow>>) {
    let window = window_q.single();
    if window.physical_width() == renderer.config.width
        && window.physical_height() == renderer.config.height
    {
        return;
    }

    renderer.resize(window.physical_width(), window.physical_height());
}
fn render_begin(
    mut renderer: ResMut<Renderer>,
    clear_color: Option<Res<ClearColor>>,
    mut error_event: EventWriter<SurfaceErrorEvent>,
) {
    let texture = match renderer.surface.get_current_texture() {
        Ok(o) => o,
        Err(e) => {
            error_event.send(SurfaceErrorEvent(e));
            return;
        }
    };
    let view = texture.texture.create_view(&TextureViewDescriptor {
        label: Some("Create surface texture view"),
        ..Default::default()
    });
    let mut encoder = renderer
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Create command encoder"),
        });

    // SAFETY: Getting static lifetimed reference from Boxed value.
    // WARNING: SURFACE MUST BE DROPPED BEFORE ANY OF THE BOXED TYPES DO
    let render_pass = encoder
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("Begin render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: match clear_color {
                        Some(clear_color) => LoadOp::Clear(clear_color.0),
                        None => LoadOp::Load,
                    },
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &renderer.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        })
        .forget_lifetime();

    renderer.render_pass = Some(RenderPassContainer {
        texture,
        view,
        encoder,
        render_pass,
    });
}
fn render_end(mut renderer: ResMut<Renderer>) {
    let Some(RenderPassContainer {
        encoder,
        texture,
        render_pass,
        view,
    }) = renderer.render_pass.take()
    else {
        return;
    };

    drop(render_pass); // NOTE: MUST BE DROPPED FIRST!

    let cmd_buf = encoder.finish();
    renderer.queue.submit(std::iter::once(cmd_buf));

    texture.present();
    drop(view); // when i leave 'view' out of the unpacking, rust seems to drop it before render_pass, so i need to explicitly declare where to drop.
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderSystem {
    Begin,
    End,
    OnResize,
    HandleSurfaceError,
}
pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_resize.in_set(RenderSystem::OnResize));
        app.add_systems(
            PostUpdate,
            (
                render_begin
                    .run_if(contains_resource::<Renderer>)
                    .in_set(RenderSystem::Begin),
                handle_surface_error
                    .after(render_begin)
                    .in_set(RenderSystem::HandleSurfaceError),
                render_end
                    .after(handle_surface_error)
                    .run_if(contains_resource::<Renderer>)
                    .in_set(RenderSystem::End),
            ),
        );
        app.init_resource::<Events<SurfaceErrorEvent>>();
        app.init_resource::<Renderer>();

        app.add_plugins((CameraPlugin, ModelPlugin));
    }
}
