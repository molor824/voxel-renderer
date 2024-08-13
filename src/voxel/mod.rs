pub mod buffer;
pub mod pipeline;

pub use buffer::*;
pub use pipeline::*;

use crate::*;

#[derive(Bundle)]
pub struct VoxelBundle {
    pub voxel: Voxel,
    pub per_instance_bind_group: PerInstanceBindGroup,
    pub model_buffer: ModelBuffer,
    pub voxel_buffer: VoxelBuffer,
    pub transform: TransformBundle,
}
impl VoxelBundle {
    pub fn new(dimension: UVec3, renderer: &Renderer, pipeline: &Pipeline) -> Self {
        let model_buffer = ModelBuffer::new(renderer);
        let voxel_buffer = VoxelBuffer::new(renderer, dimension);
        Self {
            voxel: Voxel::new(dimension),
            transform: TransformBundle::IDENTITY,
            per_instance_bind_group: PerInstanceBindGroup::new(
                renderer,
                pipeline,
                &model_buffer,
                &voxel_buffer,
            ),
            model_buffer,
            voxel_buffer,
        }
    }
}

pub struct VoxelPlugin;
impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainVoxelColors>()
            .init_resource::<MainColorBuffer>()
            .init_resource::<Pipeline>()
            .init_resource::<PerRenderBindGroup>();

        app.add_systems(
            PostUpdate,
            (
                (sync_color_buffer, sync_voxel_buffers).before(RenderSystem::Begin),
                draw.after(RenderSystem::Begin).before(RenderSystem::End),
            ),
        );
    }
}
