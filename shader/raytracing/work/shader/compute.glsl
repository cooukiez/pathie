# version 430
# extension GL_ARB_arrays_of_arrays : enable
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable

layout(local_size_x = 16, local_size_y = 16) in;

writeonly uniform image2D outImage;

struct BasicVoxel {
	bool valid;
	ivec3 position;
	vec4 color;
};

BasicVoxel[16][16][16] voxelMatrix;
vec4 color;

float getDistanceNext(float input_float) { return floor(input_float + 1) - input_float; }

BasicVoxel castRay(ivec2 dir, vec3 origin) {
	vec3 unit = vec3(cos(dir[0]), sin(dir[0]), 0.0);
	bool hit = false;
	float len = 0;
	vec3 cur_pos = origin;
	BasicVoxel voxel = BasicVoxel(false, ivec3(0, 0, 0), vec4(100, 0, 0, 0));
	color = vec4(0.0, 50.0, 0.0, 0.0);
	while (!hit) {
		vec3 timeMax = vec3(getDistanceNext(cur_pos[0]), getDistanceNext(cur_pos[1]), 0.0);
		vec3 timeMaxLength = vec3(timeMax[0] / unit[0], timeMax[1] / unit[1], 0.0);
		if (timeMaxLength[0] > timeMaxLength[1]) {
			cur_pos = vec3(unit[0] * timeMaxLength[0], unit[1] * timeMaxLength[0], 0.0);
			len += timeMaxLength[0];
		}
		if (timeMaxLength[0] < timeMaxLength[1]) {
			cur_pos = vec3(unit[0] * timeMaxLength[1], unit[1] * timeMaxLength[1], 0.0);
			len += timeMaxLength[1];
		}
		if (len > 100) { hit = true; }
		BasicVoxel voxel_at_pos = voxelMatrix[int(cur_pos[0])][int(cur_pos[1])][int(cur_pos[2])];
		if (voxel_at_pos.valid == true) { hit = true; voxel = voxel_at_pos; }
	}
	return voxel;
}

void main() {
	voxelMatrix[0][0][0] = BasicVoxel(true, ivec3(0, 0, 0), vec4(0, 0, 0, 0));
	
	BasicVoxel hit = castRay(ivec2(0, 0), vec3(0.0, 0.0, 0.0));
	imageStore(outImage, ivec2(gl_GlobalInvocationID.xy), color);
}