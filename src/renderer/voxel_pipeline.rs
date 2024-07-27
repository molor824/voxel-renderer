use bevy::math::*;
use bevy::prelude::*;
use std::mem::{size_of, MaybeUninit};
use std::num::NonZeroU64;
use std::ops::{Index, IndexMut};
use wgpu::util::*;
use wgpu::*;

use crate::*;

use super::camera::*;
use super::model::*;
use super::renderer::*;

// corners are ordered from top to bottom, in ccw
const VERTICES: [[f32; 3]; 8] = [
    [0.5, 0.5, 0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, 0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, -0.5, -0.5],
    [-0.5, -0.5, -0.5],
    [-0.5, -0.5, 0.5],
];
// indices are expected the cube to be drawn in triangle strip mode
const INDICES: [u32; 14] = [1, 2, 0, 3, 7, 2, 6, 1, 5, 0, 4, 7, 5, 6];

#[derive(Resource, Deref)]
pub struct VoxelVertexBuffer(Buffer);
impl VoxelVertexBuffer {
    pub fn new(renderer: &Renderer) -> Self {
        Self(renderer.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Create voxel vertex buffer"),
            contents: bytemuck::bytes_of(&VERTICES),
            usage: BufferUsages::VERTEX,
        }))
    }
}
impl FromWorld for VoxelVertexBuffer {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource())
    }
}

#[derive(Resource, Deref)]
pub struct VoxelIndexBuffer(Buffer);
impl VoxelIndexBuffer {
    pub fn new(renderer: &Renderer) -> Self {
        Self(renderer.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Create voxel index buffer"),
            contents: bytemuck::bytes_of(&INDICES),
            usage: BufferUsages::INDEX,
        }))
    }
}
impl FromWorld for VoxelIndexBuffer {
    fn from_world(world: &mut World) -> Self {
        Self::new(world.resource())
    }
}

#[derive(Resource, Deref)]
pub struct VoxelBindLayout(BindGroupLayout);
impl VoxelBindLayout {
    pub fn new(renderer: &Renderer) -> Self {
        Self(
            renderer
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Voxel bind group layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
        )
    }
}
impl FromWorld for VoxelBindLayout {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();
        Self::new(renderer)
    }
}

#[derive(Resource, Deref)]
struct VoxelPipeline(RenderPipeline);
impl FromWorld for VoxelPipeline {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();

        let camera_layout = &**world.resource::<CameraBindLayout>();
        let model_layout = &**world.resource::<ModelBindLayout>();
        let voxel_layout = &**world.resource::<VoxelBindLayout>();

        let shader_module = renderer
            .device
            .create_shader_module(include_wgsl!("voxel_shader.wgsl"));
        let pipeline_layout = renderer
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Voxel pipeline layout"),
                bind_group_layouts: &[camera_layout, model_layout, voxel_layout],
                push_constant_ranges: &[],
            });
        Self(
            renderer
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("Voxel render pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: VertexState {
                        module: &shader_module,
                        entry_point: "v_main",
                        buffers: &[VertexBufferLayout {
                            array_stride: size_of::<[f32; 3]>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &vertex_attr_array![0 => Float32x3],
                        }],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(FragmentState {
                        module: &shader_module,
                        entry_point: "f_main",
                        compilation_options: Default::default(),
                        targets: &[Some(ColorTargetState {
                            format: renderer.config.format,
                            write_mask: ColorWrites::ALL,
                            blend: Some(BlendState::ALPHA_BLENDING),
                        })],
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleStrip,
                        strip_index_format: Some(IndexFormat::Uint32),
                        front_face: FrontFace::Ccw,
                        cull_mode: Some(Face::Back),
                        polygon_mode: PolygonMode::Fill,
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Less,
                        stencil: StencilState::default(),
                        bias: DepthBiasState::default(),
                    }),
                    multisample: MultisampleState::default(),
                    multiview: None,
                    cache: None,
                }),
        )
    }
}

#[derive(Component)]
pub struct ColorPalette([[u8; 4]; 256]);
impl ColorPalette {
    // Color palette that contains every color of RGBA channels where each channel has 2bits
    pub fn all_color() -> Self {
        #[allow(invalid_value)]
        let mut itself: Self = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, color) in itself.0.iter_mut().enumerate() {
            color[0] = (i as u8 & 0b11) * 85;
            color[1] = (i as u8 >> 2 & 0b11) * 85;
            color[2] = (i as u8 >> 4 & 0b11) * 85;
            color[3] = (i as u8 >> 6 & 0b11) * 85;
        }
        itself
    }
}

#[derive(Component)]
pub struct VoxelBuffer {
    buffer: Buffer,
    group: BindGroup,
    dimension: UVec3,
}
impl VoxelBuffer {
    pub fn new(renderer: &Renderer, layout: &VoxelBindLayout, dimension: UVec3) -> Self {
        let buffer = renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Voxel buffer"),
            size: (dimension.x * dimension.y * dimension.z) as u64 + size_of::<UVec4>() as u64, // despite dimension being uvec3, on the gpu side, its alignment is 16 so there is an extra padding.
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Voxel bind group"),
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

        let mut view = renderer
            .queue
            .write_buffer_with(
                &buffer,
                0,
                NonZeroU64::new(size_of::<UVec3>() as u64).unwrap(),
            )
            .unwrap();

        view.as_mut()
            .clone_from_slice(bytemuck::bytes_of(&dimension));

        drop(view);

        Self {
            buffer,
            group,
            dimension,
        }
    }
    pub fn update(&self, renderer: &Renderer, voxel: &Voxel) {
        if voxel.dimension() != self.dimension {
            panic!("Attempted to update buffer with voxel whose dimension does not match.");
        }

        let mut view = renderer
            .queue
            .write_buffer_with(
                &self.buffer,
                0,
                NonZeroU64::new((size_of::<UVec4>() + voxel.len()) as u64).unwrap(),
            )
            .unwrap();

        let start = size_of::<UVec4>(); // due to buffer alignment
        let end = start + voxel.len();
        view[start..end].clone_from_slice(&voxel.data);
    }
}
#[derive(Component, Clone)]
pub struct Voxel {
    width: usize,
    height: usize,
    length: usize,
    data: Box<[u8]>,
}
impl Voxel {
    pub fn new(width: usize, height: usize, length: usize) -> Self {
        Self {
            width,
            height,
            length,
            data: vec![0; width * height * length].into_boxed_slice(),
        }
    }
    pub const fn width(&self) -> usize {
        self.width
    }
    pub const fn height(&self) -> usize {
        self.height
    }
    pub const fn length(&self) -> usize {
        self.length
    }
    pub const fn len(&self) -> usize {
        self.data.len()
    }
    pub const fn dimension(&self) -> UVec3 {
        uvec3(self.width as u32, self.height as u32, self.length as u32)
    }
    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= self.width || y >= self.height || z >= self.length {
            return None;
        }
        Some(x + y * self.width + z * self.width * self.height)
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&u8> {
        unsafe { self.index(x, y, z).map(|i| self.data.get_unchecked(i)) }
    }
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut u8> {
        unsafe { self.index(x, y, z).map(|i| self.data.get_unchecked_mut(i)) }
    }
}
impl Index<[usize; 3]> for Voxel {
    type Output = u8;
    fn index(&self, index: [usize; 3]) -> &Self::Output {
        match self.get(index[0], index[1], index[2]) {
            Some(r) => r,
            None => panic!(
                "Indices {:?} are out of bounds. Voxel dimension: {:?}",
                index,
                self.dimension()
            ),
        }
    }
}
impl IndexMut<[usize; 3]> for Voxel {
    fn index_mut(&mut self, index: [usize; 3]) -> &mut Self::Output {
        let dimension = self.dimension();
        match self.get_mut(index[0], index[1], index[2]) {
            Some(r) => r,
            None => panic!(
                "Indices {:?} are out of bounds. Voxel dimension: {:?}",
                index, dimension
            ),
        }
    }
}

fn draw(
    mut renderer: ResMut<Renderer>,
    pipeline: Res<VoxelPipeline>,
    main_camera: Res<MainCamera>,
    vertex_buffer: Res<VoxelVertexBuffer>,
    index_buffer: Res<VoxelIndexBuffer>,
    camera_q: Query<&CameraBuffer>,
    voxel_q: Query<(&VoxelBuffer, &ModelBuffer)>,
) {
    let Some(RenderPassContainer { render_pass, .. }) = &mut renderer.render_pass else {
        return;
    };

    if let Ok(camera) = camera_q.get(**main_camera) {
        render_pass.set_bind_group(0, &camera.group, &[]);
    }

    render_pass.set_pipeline(&pipeline);

    for (voxel, model) in voxel_q.iter() {
        render_pass.set_bind_group(1, &model.group, &[]);
        render_pass.set_bind_group(2, &voxel.group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
    }
}
fn sync_buffers(renderer: Res<Renderer>, voxel_q: Query<(Ref<Voxel>, &VoxelBuffer)>) {
    for (voxel, buffer) in voxel_q.iter() {
        if !voxel.is_changed() {
            continue;
        }
        buffer.update(&renderer, &voxel);
    }
}

pub struct VoxelPlugin;
impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                sync_buffers.before(RendererSystem::RenderBegin),
                draw.after(RendererSystem::RenderBegin)
                    .before(RendererSystem::RenderEnd),
            )
                .run_if(contains_resource::<Renderer>),
        );
    }
    fn finish(&self, app: &mut App) {
        app.init_resource::<VoxelBindLayout>();
        app.init_resource::<VoxelPipeline>();
        app.init_resource::<VoxelVertexBuffer>();
        app.init_resource::<VoxelIndexBuffer>();
    }
}

#[derive(Bundle)]
pub struct VoxelBundle {
    pub voxel: Voxel,
    pub voxel_buffer: VoxelBuffer,
    pub model_buffer: ModelBuffer,
    pub transform: TransformBundle,
}
impl VoxelBundle {
    pub fn new(
        renderer: &Renderer,
        voxel_layout: &VoxelBindLayout,
        model_layout: &ModelBindLayout,
        dimension: UVec3,
    ) -> Self {
        Self {
            voxel: Voxel::new(
                dimension.x as usize,
                dimension.y as usize,
                dimension.z as usize,
            ),
            voxel_buffer: VoxelBuffer::new(renderer, voxel_layout, dimension),
            model_buffer: ModelBuffer::new(renderer, model_layout),
            transform: TransformBundle::IDENTITY,
        }
    }
}
