struct VertexIn {
    @location(0) vertex: vec3<f32>,
};
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) vertex_pos: vec3<f32>,
};

struct Transform {
    projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> u_transform: Transform;

@vertex
fn v_main(input: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = u_transform.projection * vec4(input.vertex, 1.0);
    out.vertex_pos = input.vertex;
    return out;
}

// align: 16
struct Voxel {
    dimension: vec3<u32>, // offset: 0
    voxel: array<u32>,    // offset: 16
};

const LEN = 256 / 4;

@group(1) @binding(0)
var<uniform> u_colors: array<vec4<u32>, LEN>;
@group(2) @binding(0)
var<storage, read_write> s_voxel: Voxel;

fn color_from_int(integer: u32) -> vec4<f32> {
    return vec4(
        f32(integer & 0xff) / 255.0,
        f32(integer >> 8 & 0xff) / 255.0,
        f32(integer >> 16 & 0xff) / 255.0,
        f32(integer >> 24 & 0xff) / 255.0,
    );
}
fn get_color(index: u32) -> u32 {
    if index >= 256 {
        return 0u;
    }
    return u_colors[index / 4][index % 4];
}
fn x_plane_intersect(point: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    var plane_x: f32;
    if point.x > 0.0 {
        plane_x = round(point.x + 1.0);
    } else {
        plane_x = round(point.x - 1.0);
    }

    let plane_distance = plane_x - point.x;
    let distance = plane_distance / direction.x;
    
    return point + direction * distance;
}
fn y_plane_intersect(point: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    var plane_y: f32;
    if point.y > 0.0 {
        plane_y = round(point.y + 1.0);
    } else {
        plane_y = round(point.y - 1.0);
    }

    let plane_distance = plane_y - point.y;
    let distance = plane_distance / direction.y;
    
    return point + direction * distance;
}
fn z_plane_intersect(point: vec3<f32>, direction: vec3<f32>) -> vec3<f32> {
    var plane_z: f32;
    if point.z > 0.0 {
        plane_z = round(point.z + 1.0);
    } else {
        plane_z = round(point.z - 1.0);
    }

    let plane_distance = plane_z - point.z;
    let distance = plane_distance / direction.z;
    
    return point + direction * distance;
}

@fragment
fn f_main(input: VertexOut) -> @location(0) vec4<f32> {
    return vec4(input.vertex_pos, 1.0);
}