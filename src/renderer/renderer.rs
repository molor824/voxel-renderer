use std::ops::Range;

use crate::*;
use bevy::app::AppExit;
use bevy::transform::TransformSystem;
use bevy::utils::HashMap;
use bevy::window::{PrimaryWindow, RawHandleWrapper, WindowResized};
use wgpu::*;

#[derive(Component, Deref)]
pub struct RenderPipeline(pub wgpu::RenderPipeline);
#[derive(Component, Deref)]
pub struct Buffer(pub wgpu::Buffer);
#[derive(Component, Deref)]
pub struct BindGroup(pub wgpu::BindGroup);
#[derive(Component)]
pub struct Material {
    pub pipeline: Entity,
    pub bind_groups: HashMap<usize, Entity>,
}
#[derive(Component)]
pub struct Mesh {
    pub material: Entity,
    pub vertex_buffers: HashMap<usize, Entity>,
    pub index_buffer: Option<Entity>,
    pub vertex_range: Range<u32>,
    pub instance_range: Range<u32>,
}

#[derive(Resource)]
pub struct Renderer {
    pub(super) instance: Instance,
    pub(super) surface: Surface<'static>,
    pub(super) adapter: Adapter,
    pub(super) config: SurfaceConfiguration,
    pub(super) device: Device,
    pub(super) queue: Queue,
}
impl Renderer {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 && height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        info!("Surface resized to {}x{}", width, height);
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
            ..Default::default()
        });

        let mut handle_q =
            world.query_filtered::<(&RawHandleWrapper, &Window), With<PrimaryWindow>>();
        let (raw_handle, window) = handle_q.single(world);
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
            ..Default::default()
        }))
        .expect("No suitable adapters found.");

        let config = surface
            .get_default_config(&adapter, window.physical_width(), window.physical_height())
            .expect("The surface is not supported by this adapter.");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Request device"),
                ..Default::default()
            },
            None,
        ))
        .unwrap();

        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            adapter,
            config,
            device,
            queue,
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
                app_exit_event.send(AppExit);
                return;
            }
            e => error!("Surface error: {}", e),
        }
    }
}
fn on_resize(
    mut renderer: ResMut<Renderer>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut resize_event: EventReader<WindowResized>,
) {
    for event in resize_event.read() {
        match window_q.get(event.window) {
            Ok(window) => {
                renderer.resize(window.physical_width(), window.physical_height());
                return;
            }
            Err(_) => {}
        }
    }
}
fn render(
    renderer: ResMut<Renderer>,
    clear_color: Option<Res<ClearColor>>,
    mut error_event: EventWriter<SurfaceErrorEvent>,
    buffer_q: Query<&Buffer>,
    render_pipeline_q: Query<&RenderPipeline>,
    material_q: Query<&Material>,
    mesh_q: Query<&Mesh>,
    bind_group_q: Query<&BindGroup>,
) {
    let output = match renderer.surface.get_current_texture() {
        Ok(o) => o,
        Err(e) => {
            error_event.send(SurfaceErrorEvent(e));
            return;
        }
    };
    let view = output.texture.create_view(&TextureViewDescriptor {
        label: Some("Create surface texture view"),
        ..Default::default()
    });
    let mut encoder = renderer
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Create command encoder"),
        });

    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
        ..Default::default()
    });

    for mesh in mesh_q.iter() {
        let material = material_q.get(mesh.material).unwrap();
        let pipeline = render_pipeline_q.get(material.pipeline).unwrap();

        render_pass.set_pipeline(pipeline);

        for (&slot, &entity) in mesh.vertex_buffers.iter() {
            let buffer = buffer_q.get(entity).unwrap();
            render_pass.set_vertex_buffer(slot as u32, buffer.slice(..));
        }
        for (&index, &entity) in material.bind_groups.iter() {
            let group = bind_group_q.get(entity).unwrap();
            render_pass.set_bind_group(index as u32, group, &[]);
        }

        if let Some(entity) = mesh.index_buffer {
            let buffer = buffer_q.get(entity).unwrap();
            render_pass.set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(mesh.vertex_range.clone(), 0, mesh.instance_range.clone());
        } else {
            render_pass.draw(mesh.vertex_range.clone(), mesh.instance_range.clone());
        }
    }

    drop(render_pass);

    let command_buffer = encoder.finish();

    renderer.queue.submit(std::iter::once(command_buffer));
    output.present();
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<SurfaceErrorEvent>>();
        app.add_systems(Update, on_resize);
        app.add_systems(
            PostUpdate,
            (
                render
                    .after(TransformSystem::TransformPropagate)
                    .run_if(contains_resource::<Renderer>),
                handle_surface_error.after(render),
            ),
        );
    }
    fn finish(&self, app: &mut App) {
        app.init_resource::<Renderer>();
    }
}
