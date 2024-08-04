use std::mem::size_of;

use wgpu::*;

use crate::*;

#[derive(Resource, Deref)]
pub struct TransformBindLayout(BindGroupLayout);
impl TransformBindLayout {
    fn new(renderer: &Renderer) -> Self {
        Self(
            renderer
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Transform bind group layout"),
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
impl FromWorld for TransformBindLayout {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource())
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TransformValue {
    projection: Mat4,
    camera_pos: Vec3,
}
unsafe impl bytemuck::NoUninit for TransformValue {}

#[derive(Component)]
pub struct TransformBuffer {
    pub buffer: Buffer,
    pub group: BindGroup,
}
impl TransformBuffer {
    const BUFFER_SIZE: u64 = size_of::<Mat4>() as u64 + size_of::<Vec4>() as u64;
    pub fn new(renderer: &Renderer, layout: &TransformBindLayout) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Transform buffer"),
            size: Self::BUFFER_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Transform bind group"),
            layout,
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
    fn update(&self, renderer: &Renderer, value: &TransformValue) {
        renderer.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }
}

fn sync_buffers(
    renderer: Res<Renderer>,
    main_camera: Res<MainCamera>,
    window_q: Query<Ref<Window>, With<PrimaryWindow>>,
    camera_q: Query<(Ref<Camera>, Ref<GlobalTransform>)>,
    model_q: Query<(Ref<GlobalTransform>, &TransformBuffer)>,
) {
    let (camera, cam_transform) = camera_q.get(**main_camera).expect("Main camera not found!");
    let window = window_q.single();
    let camera_changed = camera.is_changed() || cam_transform.is_changed() || window.is_changed();
    let aspect = window.physical_width() as f32 / window.physical_height() as f32;

    let cam_matrix = camera.projection(aspect) * cam_transform.compute_matrix().inverse();

    for (transform, buffer) in model_q.iter() {
        if !transform.is_changed() && !camera_changed {
            continue;
        }
        let projection = cam_matrix * transform.compute_matrix();
        let value = TransformValue {
            projection,
            camera_pos: (*transform * *cam_transform).translation(),
        };
        buffer.update(&renderer, &value);
    }
}

pub struct ModelPlugin;
impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransformBindLayout>();

        app.add_systems(
            PostUpdate,
            sync_buffers
                .run_if(contains_resource::<Renderer>)
                .before(RendererSystem::RenderBegin),
        );
    }
}
