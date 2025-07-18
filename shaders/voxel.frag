#version 450

layout(location = 0) in vec3 i_point;
layout(location = 1) flat in vec3 i_camera_pos;
layout(location = 2) flat in uint i_iterations;
layout(location = 3) flat in vec3 i_normal;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec3 frag_normal;

layout(set = 0, binding = 1, std430) readonly buffer Voxel {
    uvec4 dimension;
    uint voxels[];
} voxel;
layout(set = 1, binding = 1, std140) uniform Colors {
    uvec4 colors[64];
};

struct HitInfo {
    vec3 intersection;
    vec3 normal;
    uvec3 voxel_pos;
};

float intersect_plane(vec3 point, vec3 direction, uint axis) {
    float plane = direction[axis] > 0 ? ceil(point[axis]) : floor(point[axis]);
    float dist = abs(plane - point[axis]) / abs(direction[axis]);
    return dist;
}
HitInfo intersect_nearest(vec3 point, vec3 direction) {
    float distances[3] = {
        intersect_plane(point, direction, 0),
        intersect_plane(point, direction, 1),
        intersect_plane(point, direction, 2)
    };
    uint index = 0;
    if (distances[index] > distances[1] || isnan(distances[index]) || isinf(distances[index])) index = 1;
    if (distances[index] > distances[2] || isnan(distances[index]) || isinf(distances[index])) index = 2;
    
    float distance = distances[index];
    vec3 intersection = point + direction * distance;
    vec3 normal = vec3(0.0);
    normal[index] = -sign(direction[index]);
    uvec3 voxel_pos = uvec3(ivec3(floor(intersection - normal * 0.5)) + ivec3(voxel.dimension) / 2);

    return HitInfo(intersection, normal, voxel_pos);
}
uint get_voxel_color(uvec3 voxel_pos) {
    uvec3 dimension = voxel.dimension.xyz;
    if (voxel_pos.x >= dimension.x || voxel_pos.y >= dimension.y || voxel_pos.z >= dimension.z)
        return 0;
    uint index = voxel_pos.x + voxel_pos.y * dimension.x + voxel_pos.z * dimension.x * dimension.y;
    uint color_index = (voxel.voxels[index / 4] >> ((index % 4) * 8)) & 0xff;
    return colors[color_index / 4][color_index % 4];
}
vec4 unpack_color(uint packed) {
    return vec4(
        float(packed & 0xff) / 255.0,
        float((packed >> 8) & 0xff) / 255.0,
        float((packed >> 16) & 0xff) / 255.0,
        float((packed >> 24) & 0xff) / 255.0
    );
}

const float THRESHOLD = 0.0001;
const vec3 LIGHT_DIR = normalize(vec3(-3.0, -10.0, -5.0));

void main() {
    vec3 direction = normalize(i_point - i_camera_pos);
    float direction_dot = dot(direction, i_normal);

    vec3 point = (direction_dot < 0.0 ? i_point : i_camera_pos) * vec3(voxel.dimension);
    vec4 color = vec4(0.0);
    vec3 normal = vec3(0.0);

    point -= direction * THRESHOLD;

    for (uint i = 0; i < i_iterations; i++) {
        HitInfo info = intersect_nearest(point, direction);
        vec4 hit_color = unpack_color(get_voxel_color(info.voxel_pos));
        if (color.w < 1.0) {
            normal = info.normal;
        }
        vec3 cf = color.xyz; // foreground color
        float af = color.w; // foreground alpha
        vec3 cb = hit_color.xyz; // background color
        float ab = hit_color.w; // background alpha
        vec3 cr = cf * af + cb * (1.0 - af); // alpha-blending result rgb color
        float ar = af + ab * (1.0 - af); // alpha-blending result alpha channel
        color = vec4(cr, ar);
        point = info.intersection + direction * THRESHOLD;
    }

    float light_dot = (dot(normal, -LIGHT_DIR) + 1) * 0.5;
    frag_normal = normal;
    frag_color = color * light_dot;
}
