# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

layout (set = 0, binding = 0) uniform Uniform {
    uint time;

	float rawfieldOfView;
	uint maxRayLen;

	float rotHorizontal;
	float rotVertical;

	uint octreeRootIndex;
	
	uint nodeAtPos;
	float X;
	float Y;
	float Z;
} uniformBuffer;

struct TreeNode {
    uint mat;
	uint parent;

    uint span;
    uint spaceIndex;

	uint[8] children;
	
	float X;
	float Y;
	float Z;
}

layout (location = 0) in vec2 outCoord;
layout (location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(int(outCoord.y * 2.0),int(sin(outCoord.x * 2.0) * 10),0,0);
    
    if (outCoord.x < 0.001 && outCoord.y < 0.001) {
		debugPrintfEXT("\n%f", uniformBuffer.rawfieldOfView);
	}
}
