#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
//#extension GL_EXT_scalar_block_layout : enable
#extension GL_EXT_debug_printf : enable

layout (location = 0) in vec4 inPos;
layout (location = 1) in vec2 inCoord;

layout (location = 0) out vec2 localPos;
layout (location = 1) out vec4 outPos;

struct PosInfo {
    vec4 local_pos;
    vec4 pos_on_edge;
    uint depth;

    uint padding[3];
};

layout (set = 0, binding = 0) uniform Uniform {
    mat4 view_proj;
    vec4 pos;

    vec2 res;
    vec2 mouse_delta;
    vec2 mouse_pos;
    vec2 rot;

    float root_span;
    uint time;

    uint padding[2];

    PosInfo pos_info;
} uniform_buffer;

void main() {
    localPos = inCoord;
    gl_Position = uniform_buffer.view_proj * inPos;
    outPos = gl_Position;
}
