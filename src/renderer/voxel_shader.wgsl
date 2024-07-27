struct VertexIn {
    @location(0) vertex: vec3<f32>,
};
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) pos: vec3<f32>,
};

struct CameraTransform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> u_camera: CameraTransform;

@group(1) @binding(0)
var<uniform> u_model: mat4x4<f32>;

@vertex
fn v_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = u_camera.projection * u_camera.view * u_model * vec4<f32>(input.vertex, 1.0);
    out.pos = input.vertex;
    return out;
}

// align: 16
struct Voxel {
    dimension: vec3<u32>, // offset: 0
    voxel: array<u32>,    // offset: 16
};

const len = 256 / 4;

@group(2) @binding(0)
var<storage> s_voxel: Voxel;
// @group(3) @binding(0)
// var<uniform> u_colors: array<vec4<u32>, len>;

// fn color_from_int(integer: u32) -> vec4<f32> {
//     return vec4<f32>(
//         f32(integer & 0xff) / 255.0,
//         f32(integer >> 8 & 0xff) / 255.0,
//         f32(integer >> 16 & 0xff) / 255.0,
//         f32(integer >> 24 & 0xff) / 255.0,
//     );
// }
// fn get_color(index: u32) -> u32 {
//     if index >= 256 {
//         return 0u;
//     }
//     return u_colors[index / 4][index % 4];
// }

@fragment
fn f_main(input: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(input.pos, 0.5);
}