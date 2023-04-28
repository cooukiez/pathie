#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable
#extension GL_EXT_scalar_block_layout : enable

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

layout (std430, set = 0, binding = 0) uniform Uniform {
    vec4 pos;

    vec2 res;
    vec2 mouse;

    float rootSpan;

    uint time;
} uniformBuffer;

struct TreeNode {
    uint children[8];
    
    // 0 = empty | 1 = subdivide | 2 = full
    uint nodeType;
	uint parent;
    vec4 baseColor; // ToDo -> Add transparency
};

layout (std430, set = 1, binding = 0) buffer OctreeData { TreeNode octreeData[102400]; };

struct Light {
    vec4 pos;
    uint index;
};

layout (set = 2, binding = 0) readonly buffer LightData { Light lightData[2048]; };

void main() {
    fragColor = vec4(0.0, 0.0, 1.0, 1.0);
}
