use std::{mem::size_of, num::NonZeroU64};

use wgpu::*;

use crate::*;

#[derive(Resource, Deref)]
pub struct ModelBindLayout(pub BindGroupLayout);
impl FromWorld for ModelBindLayout {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();
        Self(ModelBuffer::create_layout(renderer))
    }
}

#[derive(Component)]
pub struct ModelBuffer {
    pub buffer: Buffer,
    pub group: BindGroup,
}
impl ModelBuffer {
    pub fn new(renderer: &Renderer, layout: &ModelBindLayout) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Model buffer"),
            size: size_of::<Mat4>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Model bind group"),
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
    pub fn update(&self, renderer: &Renderer, transform: &GlobalTransform) {
        let mut view = renderer
            .queue
            .write_buffer_with(
                &self.buffer,
                0,
                NonZeroU64::new(size_of::<Mat4>() as u64).unwrap(),
            )
            .unwrap();
        view.clone_from_slice(bytemuck::bytes_of(&transform.compute_matrix()));
    }
    fn create_layout(renderer: &Renderer) -> BindGroupLayout {
        renderer
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Model bind group layout"),
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
            })
    }
}

fn sync_buffers(renderer: Res<Renderer>, model_q: Query<(Ref<GlobalTransform>, &ModelBuffer)>) {
    for (transform, buffer) in model_q.iter() {
        if !transform.is_changed() {
            continue;
        }
        buffer.update(&renderer, &transform);
    }
}

pub struct ModelPlugin;
impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_buffers.run_if(contains_resource::<Renderer>).before(RendererSystem::RenderBegin));
    }
    fn finish(&self, app: &mut App) {
        app.init_resource::<ModelBindLayout>();
    }
}
