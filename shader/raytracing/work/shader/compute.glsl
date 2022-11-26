# version 430
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable

# define MAX 100
# define PI 3.141592654
# define LIMIT 2
# define ACCURACY 100
# define FOV 60.0

layout(local_size_x = 16, local_size_y = 16) in;

writeonly uniform image2D outImage;

void main () {
	float degree = (FOV / gl_NumWorkGroups.x) * gl_GlobalInvocationID.x;

	imageStore(outImage, ivec2(gl_GlobalInvocationID.xy), vec4(0,0,degree,0));
}