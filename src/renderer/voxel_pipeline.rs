use crate::renderer::{self, Renderer};

use bevy::prelude::*;
use std::mem::size_of;
use wgpu::util::*;
use wgpu::*;

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

#[derive(Component)]
pub struct VoxelMaterial;
#[derive(Component)]
pub struct VoxelMesh;

fn create_index_buffer(renderer: &Renderer) -> Buffer {
    renderer.device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Create voxel index buffer"),
        contents: bytemuck::bytes_of(&INDICES),
        usage: BufferUsages::INDEX,
    })
}
fn create_vertex_buffer(renderer: &Renderer) -> Buffer {
    renderer.device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Create voxel vertex buffer"),
        contents: bytemuck::bytes_of(&VERTICES),
        usage: BufferUsages::VERTEX,
    })
}
fn create_vertex_layout() -> VertexBufferLayout<'static> {
    VertexBufferLayout {
        array_stride: size_of::<[f32; 3]>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![0 => Float32x3],
    }
}
fn create_transform_buffers(renderer: &Renderer) -> [Buffer; 3] {
    [(); 3].map(|_| {
        renderer.device.create_buffer(&BufferDescriptor {
            label: Some("Transform buffer"),
            size: size_of::<[[f32; 4]; 4]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    })
}
fn create_transform_layout(renderer: &Renderer) -> BindGroupLayout {
    let entries = [0, 1, 2].map(|i| BindGroupLayoutEntry {
        binding: i,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    });
    renderer
        .device
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Transform uniform buffer bind group layout"),
            entries: &entries,
        })
}
fn create_transform_bind_group(
    renderer: &Renderer,
    layout: &BindGroupLayout,
    buffers: &[Buffer; 3],
) -> BindGroup {
    let entries = [0, 1, 2].map(|i| BindGroupEntry {
        binding: i,
        resource: BindingResource::Buffer(BufferBinding {
            buffer: &buffers[i as usize],
            offset: 0,
            size: None,
        }),
    });
    renderer.device.create_bind_group(&BindGroupDescriptor {
        label: Some("Transform uniform buffer bind group"),
        layout,
        entries: &entries,
    })
}
fn create_render_pipeline(
    renderer: &Renderer,
    transform_layout: &BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = renderer
        .device
        .create_shader_module(include_wgsl!("voxel_shader.wgsl"));
    let pipeline_layout = renderer
        .device
        .create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Voxel pipeline layout"),
            bind_group_layouts: &[transform_layout],
            push_constant_ranges: &[],
        });
    let pipeline = renderer
        .device
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Voxel render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "v_main",
                buffers: &[create_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "f_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: renderer.config.format,
                    write_mask: ColorWrites::ALL,
                    blend: Some(BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

    pipeline
}
pub fn setup(mut commands: Commands, renderer: Res<Renderer>) {
    let transform_layout = create_transform_layout(&renderer);
    let pipeline_id = commands
        .spawn(renderer::RenderPipeline(create_render_pipeline(
            &renderer,
            &transform_layout,
        )))
        .id();
    let vertex_buffer_id = commands
        .spawn(renderer::Buffer(create_vertex_buffer(&renderer)))
        .id();
    let index_buffer_id = commands
        .spawn(renderer::Buffer(create_index_buffer(&renderer)))
        .id();
    let transform_buffers = create_transform_buffers(&renderer);
    let transform_bind_group_id = commands
        .spawn(renderer::BindGroup(create_transform_bind_group(
            &renderer,
            &transform_layout,
            &transform_buffers,
        )))
        .id();
    let material_id = commands.spawn((
        VoxelMaterial,
        renderer::Material {
            pipeline: pipeline_id,
            bind_groups: [(0, transform_bind_group_id)].into(),
        },
    )).id();
    commands.spawn((VoxelMesh, renderer::Mesh {
        material: material_id,
        vertex_buffers: [(0, vertex_buffer_id)].into(),
        index_buffer: Some(index_buffer_id),
        vertex_range: 0..(INDICES.len() as u32),
        instance_range: 0..1,
    }));
}

const VOXEL_SIZE: [usize; 3] = [16; 3];

#[derive(Resource)]
pub struct ColorPalette(pub [Color; 256]);

#[derive(Component)]
pub struct Voxel(pub [[[u8; VOXEL_SIZE[2]]; VOXEL_SIZE[1]]; VOXEL_SIZE[0]]);
