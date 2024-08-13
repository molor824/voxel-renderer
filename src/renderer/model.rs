use crate::*;
use bytemuck::NoUninit;
use wgpu::*;

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct ModelBufferValue {
    pub transform: Mat4,
    pub inv_transform: Mat4,
}
unsafe impl NoUninit for ModelBufferValue {}

#[derive(Component, Deref)]
pub struct ModelBuffer(Buffer);
impl ModelBuffer {
    pub fn new(renderer: &Renderer) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Model buffer"),
            size: size_of::<ModelBufferValue>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self(buffer)
    }
    fn update(&self, renderer: &Renderer, value: &ModelBufferValue) {
        renderer
            .queue
            .write_buffer(&*self, 0, bytemuck::bytes_of(value));
    }
}

fn sync_buffers(renderer: Res<Renderer>, model_q: Query<(Ref<GlobalTransform>, &ModelBuffer)>) {
    for (transform, buffer) in model_q.iter() {
        if !transform.is_changed() {
            continue;
        }
        let transform_matrix = transform.compute_matrix();
        let value = ModelBufferValue {
            transform: transform_matrix,
            inv_transform: transform_matrix.inverse(),
        };
        buffer.update(&renderer, &value);
    }
}

pub struct ModelPlugin;
impl Plugin for ModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            sync_buffers
                .run_if(contains_resource::<Renderer>)
                .before(RenderSystem::Begin),
        );
    }
}
