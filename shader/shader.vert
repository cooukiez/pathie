#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable

layout (location = 0) in vec4 inPos;
layout (location = 1) in vec2 inCoord;

layout (location = 0) out vec2 localPos;

void main() {
    localPos = inCoord;
    gl_Position = inPos;
}
