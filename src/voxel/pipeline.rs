use bevy::math::*;
use bevy::prelude::*;
use wgpu::*;

use crate::model::*;
use crate::*;

#[derive(Resource)]
pub struct Pipeline {
    pipeline: RenderPipeline,
    per_instance_layout: BindGroupLayout,
    per_render_layout: BindGroupLayout,
}
impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let renderer = world.resource::<Renderer>();

        fn create_entry(
            binding: u32,
            ty: BufferBindingType,
            visibility: ShaderStages,
        ) -> BindGroupLayoutEntry {
            BindGroupLayoutEntry {
                binding,
                visibility,
                ty: BindingType::Buffer {
                    ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        }

        // model and voxel layout
        let per_instance_layout =
            renderer
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Voxel per instance bind group layout"),
                    entries: &[
                        create_entry(0, BufferBindingType::Uniform, ShaderStages::VERTEX),
                        create_entry(
                            1,
                            BufferBindingType::Storage { read_only: true },
                            ShaderStages::VERTEX_FRAGMENT,
                        ),
                    ],
                });

        // camera and color layout
        let per_render_layout =
            renderer
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Voxel per render bind group layout"),
                    entries: &[
                        create_entry(0, BufferBindingType::Uniform, ShaderStages::VERTEX),
                        create_entry(1, BufferBindingType::Uniform, ShaderStages::FRAGMENT),
                    ],
                });

        let vert_shader_module = unsafe {
            renderer
                .device
                .create_shader_module_spirv(&include_spirv_raw!("../../target/voxel.vert.spv"))
        };
        let frag_shader_module = unsafe {
            renderer
                .device
                .create_shader_module_spirv(&include_spirv_raw!("../../target/voxel.frag.spv"))
        };

        let pipeline_layout = renderer
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Voxel pipeline layout"),
                bind_group_layouts: &[&per_instance_layout, &per_render_layout],
                push_constant_ranges: &[],
            });
        let pipeline = renderer
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Voxel render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &vert_shader_module,
                    entry_point: "main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &frag_shader_module,
                    entry_point: "main",
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format: renderer.config.format,
                        write_mask: ColorWrites::ALL,
                        blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING), // for now implement without alpha blending
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
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
            });
        Self {
            pipeline,
            per_instance_layout,
            per_render_layout,
        }
    }
}

fn create_entry(binding: u32, buffer: &Buffer) -> BindGroupEntry {
    BindGroupEntry {
        binding,
        resource: BindingResource::Buffer(BufferBinding {
            buffer,
            offset: 0,
            size: None,
        }),
    }
}

#[derive(Resource, Deref)]
pub struct PerRenderBindGroup(BindGroup);
impl PerRenderBindGroup {
    pub fn new(
        renderer: &Renderer,
        pipeline: &Pipeline,
        camera_buffer: &MainCameraBuffer,
        color_buffer: &MainColorBuffer,
    ) -> Self {
        Self(renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Voxel per render bind group"),
            layout: &pipeline.per_render_layout,
            entries: &[
                create_entry(0, &camera_buffer),
                create_entry(1, &color_buffer),
            ],
        }))
    }
}
impl FromWorld for PerRenderBindGroup {
    fn from_world(world: &mut World) -> Self {
        Self::new(
            world.resource(),
            world.resource(),
            world.resource(),
            world.resource(),
        )
    }
}

#[derive(Component, Deref)]
pub struct PerInstanceBindGroup(BindGroup);
impl PerInstanceBindGroup {
    pub fn new(
        renderer: &Renderer,
        pipeline: &Pipeline,
        model_buffer: &ModelBuffer,
        voxel_buffer: &VoxelBuffer,
    ) -> Self {
        Self(renderer.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Voxel per instance bind group"),
            layout: &pipeline.per_instance_layout,
            entries: &[
                create_entry(0, &model_buffer),
                create_entry(1, &voxel_buffer),
            ],
        }))
    }
}

pub(super) fn draw(
    mut renderer: ResMut<Renderer>,
    pipeline: Res<Pipeline>,
    per_render: Res<PerRenderBindGroup>,
    per_instance_q: Query<&PerInstanceBindGroup>,
) {
    let Some(RenderPassContainer { render_pass, .. }) = &mut renderer.render_pass else {
        return;
    };

    render_pass.set_pipeline(&pipeline.pipeline);
    render_pass.set_bind_group(1, &*per_render, &[]);

    for per_instance in per_instance_q.iter() {
        render_pass.set_bind_group(0, &*per_instance, &[]);

        render_pass.draw(0..36, 0..1);
    }
}
