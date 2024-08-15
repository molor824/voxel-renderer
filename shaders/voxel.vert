#version 450

const vec3 VERTICES[8] = {
    {0.5, 0.5, 0.5},
    {0.5, 0.5, -0.5},
    {-0.5, 0.5, -0.5},
    {-0.5, 0.5, 0.5},
    {0.5, -0.5, 0.5},
    {0.5, -0.5, -0.5},
    {-0.5, -0.5, -0.5},
    {-0.5, -0.5, 0.5},
};
const uint INDICES[14] = {1, 2, 0, 3, 7, 2, 6, 1, 5, 0, 4, 7, 5, 6};

layout(location = 0) out vec3 o_point;
layout(location = 1) flat out vec3 o_camera_pos;
layout(location = 2) flat out uint o_iterations;

layout(set = 0, binding = 0) uniform Model {
    mat4 transform;
    mat4 inv_transform;
} model;
layout(set = 0, binding = 1) readonly buffer Voxel {
    uvec3 dimension;
    uint voxels[];
} voxel;
layout(set = 1, binding = 0) uniform Camera {
    mat4 transform;
    mat4 inv_transform;
    mat4 projection;
} camera;

void main() {
    vec3 vertex = VERTICES[INDICES[gl_VertexIndex]];

    vec4 point = camera.inv_transform * model.transform * vec4(vertex, 1.0);
    vec4 camera_pos = model.inv_transform * camera.transform[3];
    o_camera_pos = camera_pos.xyz / camera_pos.w;
    o_point = point.xyz / point.w;
    gl_Position = camera.projection * point;
    
    o_iterations = max(voxel.dimension.x, max(voxel.dimension.y, voxel.dimension.z)) * 8;
}
