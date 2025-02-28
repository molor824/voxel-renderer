#version 450

const vec3 VERTICES[8] = {
    vec3(-0.5, -0.5, -0.5), // 0 - Bottom-left-back
    vec3( 0.5, -0.5, -0.5), // 1 - Bottom-right-back
    vec3( 0.5,  0.5, -0.5), // 2 - Top-right-back
    vec3(-0.5,  0.5, -0.5), // 3 - Top-left-back
    vec3(-0.5, -0.5,  0.5), // 4 - Bottom-left-front
    vec3( 0.5, -0.5,  0.5), // 5 - Bottom-right-front
    vec3( 0.5,  0.5,  0.5), // 6 - Top-right-front
    vec3(-0.5,  0.5,  0.5), // 7 - Top-left-front
};
const vec3 NORMALS[6] = {
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
};
const uint INDICES[36] = {
    // Back face
    0, 1, 2,  2, 3, 0,
    // Front face
    5, 4, 7,  7, 6, 5,
    // Left face
    4, 0, 3,  3, 7, 4,
    // Right face
    1, 5, 6,  6, 2, 1,
    // Bottom face
    4, 5, 1,  1, 0, 4,
    // Top face
    3, 2, 6,  6, 7, 3,
};

layout(location = 0) out vec3 o_point;
layout(location = 1) flat out vec3 o_camera_pos;
layout(location = 2) flat out uint o_iterations;
layout(location = 3) flat out vec3 o_normal;

layout(set = 0, binding = 0, std140) uniform Model {
    mat4 transform;
    mat4 inv_transform;
} model;
layout(set = 0, binding = 1, std430) readonly buffer Voxel {
    uvec3 dimension;
} voxel;
layout(set = 1, binding = 0, std140) uniform Camera {
    mat4 transform;
    mat4 inv_transform;
    mat4 projection;
} camera;

void main() {
    vec3 vertex = VERTICES[INDICES[gl_VertexIndex]];
    vec3 normal = NORMALS[gl_VertexIndex / 6];

    vec4 camera_pos = model.inv_transform * camera.transform[3];
    o_camera_pos = camera_pos.xyz;
    o_point = vertex;
    o_normal = normal;
    o_iterations =  voxel.dimension.x + voxel.dimension.y + voxel.dimension.z;
    gl_Position = camera.projection * camera.inv_transform * model.transform * vec4(vertex, 1.0);
}
