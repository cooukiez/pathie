#version 330

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 inImage;

out vec2 outImage;

void main() {
	gl_Position = vec4(pos, 0.0, 1.0);
	outImage = inImage;
}
