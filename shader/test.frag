#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_scalar_block_layout : enable
//#extension GL_EXT_debug_printf : enable

layout (location = 0) in vec4 screen_pos;
layout (location = 1) in vec2 out_uv;
layout (location = 2) in vec4 world_pos; // pos_on_edge + local_pos
layout (location = 3) flat in uint loc_idx;

layout (location = 0) out vec4 frag_color;

layout (set = 0, binding = 0) uniform Uniform {
    mat4 view_proj;
    vec4 pos;

    vec4 cam_pos;
    vec4 cam_front;
    vec4 cam_up;
    vec4 look_dir;

    vec2 res;
    vec2 mouse_delta;
    vec2 mouse_pos;

    float root_span;
    uint time;

    uint padding[2];
} uniform_buffer;

void main() {
    frag_color = world_pos;
}