use super::renderer::*;
use crate::*;
use bevy::window::PrimaryWindow;
use std::{mem::size_of, num::NonZeroU64};
use wgpu::*;

#[derive(Component, Clone, Copy)]
pub struct Orthographic {
    pub height_scale: f32,
    pub near: f32,
    pub far: f32,
}
impl Orthographic {
    // ratio is width / height
    pub fn projection(&self, ratio: f32) -> Mat4 {
        let width = ratio * self.height_scale;
        Mat4::orthographic_rh(
            width * -0.5,
            width * 0.5,
            self.height_scale * -0.5,
            self.height_scale * 0.5,
            self.near,
            self.far,
        )
    }
}
impl Default for Orthographic {
    fn default() -> Self {
        Self {
            height_scale: 2.0,
            near: -1.0,
            far: 1.0,
        }
    }
}
#[derive(Component, Clone, Copy)]
pub struct Perspective {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Perspective {
    pub fn projection(&self, ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, ratio, self.near, self.far)
    }
}
impl Default for Perspective {
    fn default() -> Self {
        Self {
            fov: 90.0_f32.to_radians(),
            near: 0.001,
            far: 1000.0,
        }
    }
}

#[derive(Component)]
pub struct CameraBuffer {
    pub buffer: Buffer,
    pub group: BindGroup,
}
impl CameraBuffer {
    pub fn new(renderer: &Renderer, bind_group_layout: &CameraBindLayout) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Camera buffer"),
            size: size_of::<[Mat4; 2]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Create camera buffer bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self { buffer, group }
    }
    pub fn update(
        &self,
        renderer: &Renderer,
        aspect_ratio: f32,
        camera: &Camera,
        transform: &GlobalTransform,
    ) {
        let mut buffer_view = renderer
            .queue
            .write_buffer_with(
                &self.buffer,
                0,
                NonZeroU64::new(size_of::<[Mat4; 2]>() as u64).unwrap(),
            )
            .unwrap();

        let transforms = [
            transform.compute_matrix().inverse(),
            camera.projection(aspect_ratio),
        ];
        buffer_view.clone_from_slice(bytemuck::bytes_of(&transforms));
    }
}

#[derive(Component, Clone, Copy)]
pub enum Camera {
    Orthographic(Orthographic),
    Perspective(Perspective),
}
impl Camera {
    pub fn projection(&self, ratio: f32) -> Mat4 {
        match self {
            Self::Orthographic(ortho) => ortho.projection(ratio),
            Self::Perspective(pers) => pers.projection(ratio),
        }
    }
}

#[derive(Resource, Deref)]
pub struct CameraBindLayout(BindGroupLayout);
impl CameraBindLayout {
    pub fn new(renderer: &Renderer) -> Self {
        Self(
            renderer
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Camera buffer bind group layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
        )
    }
}
impl FromWorld for CameraBindLayout {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();
        Self::new(renderer)
    }
}

#[derive(Resource, Deref)]
pub struct MainCamera(Entity);

pub fn setup(mut commands: Commands, renderer: Res<Renderer>, layout: Res<CameraBindLayout>) {
    let buffers = CameraBuffer::new(&renderer, &layout);
    let main_camera = commands.spawn((Camera::Perspective(Default::default()), buffers, TransformBundle::default())).id();
    commands.insert_resource(MainCamera(main_camera));
}
pub fn sync_buffers(
    renderer: Res<Renderer>,
    camera_q: Query<(&Camera, &CameraBuffer, &GlobalTransform)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.get_single() else {
        return;
    };
    let aspect_ratio = window.width() / window.height();

    for (camera, camera_buffer, transform) in camera_q.iter() {
        camera_buffer.update(&renderer, aspect_ratio, camera, transform);
    }
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            PostUpdate,
            sync_buffers
                .run_if(contains_resource::<Renderer>)
                .before(RendererSystem::RenderBegin),
        );
    }
    fn finish(&self, app: &mut App) {
        app.init_resource::<CameraBindLayout>();
    }
}
