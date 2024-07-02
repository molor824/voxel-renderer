struct VertexIn {
    @location(0) vertex: vec3<f32>,
};
struct VertexOut {
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u_model: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> u_view: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> u_projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> u_voxel_grid: array<u32>;
@group(1) @binding(1)
var<uniform> u_voxel_dimension: vec3<u32>;

@vertex
fn v_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = u_projection * u_view * u_model * vec4<f32>(input.vertex, 1.0);
    return out;
}

@fragment
fn f_main(input: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}