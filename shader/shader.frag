#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable

layout (location = 0) in vec2 outCoord;
layout (location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(int(outCoord.y * 2.0),int(sin(outCoord.x * 2.0) * 10),0,0);
}
