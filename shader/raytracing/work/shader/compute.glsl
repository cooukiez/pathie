# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

# define PI 3.141592654
# define LIMIT 1
# define ACCURACY 100
# define FOV 60.0

layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
writeonly uniform image2D outImage;

int intMatrix[1024][16];

void main () {
	for (int iter = 0; iter < 2048; iter += 1) {
		intMatrix[iter][0] = 1;
	}
	// intMatrix[16][0] = 1;
	ivec2 pixelCoord = ivec2(gl_GlobalInvocationID.xy);
	int test = intMatrix[pixelCoord.x][0];
	imageStore(outImage, pixelCoord, vec4(1,test,0,0));
}