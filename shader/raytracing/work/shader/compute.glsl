# version 430
# extension GL_ARB_arrays_of_arrays : enable
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable

# define MAX 100
# define PI 3.141592654
# define STEP_LIMIT 10

layout(local_size_x = 16, local_size_y = 16) in;

writeonly uniform image2D outImage;

struct BasicVoxel {
	bool valid;
	vec4 color;
};

BasicVoxel[16][16][16] voxelMatrix;

float roundFloat(float val) { return round(val * 1000) / 1000; }
float getDistanceNext(float input_float) { return floor(input_float + 1) - input_float; }
vec3 getNewPos(vec3 unit, vec3 old_pos, float selected_length) { return vec3(unit.x * selected_length + old_pos.x, unit.y * selected_length + old_pos.y, unit.z * selected_length + old_pos.z); }

float getTimeMaxLen(float timeMax, float unit) { if (unit != 0.0) { return timeMax / unit; } else { return 0.0; } }

BasicVoxel castRay(ivec2 dir, vec3 origin, int step_limit) {
	vec2 dir_radiant = vec2(roundFloat(dir.x * PI / 180), roundFloat(dir.y * PI / 180));
	vec3 unit = vec3(roundFloat(cos(dir_radiant.x) * cos(dir_radiant.y)), roundFloat(sin(dir_radiant.y)), roundFloat(sin(dir_radiant.x) * cos(dir_radiant.y)));
	float unitLen = sqrt(roundFloat(unit.x * unit.x + unit.y * unit.y + unit.z * unit.z));

	float len = 0;
	vec3 cur_pos = origin;
	BasicVoxel voxel = BasicVoxel(false, vec4(0.0, 1.0, 0.0, 0.0));
	
	for(int iteration = 0; iteration < step_limit; iteration += 1) {
		vec3 distNextVox = vec3(getDistanceNext(cur_pos.x), getDistanceNext(cur_pos.y), getDistanceNext(cur_pos.z));
		vec3 lengthCertainDir = vec3(getTimeMaxLen(distNextVox.x, unit.x), getTimeMaxLen(distNextVox.y, unit.y), getTimeMaxLen(distNextVox.z, unit.z));

		if (lengthCertainDir.x < lengthCertainDir.y) {
			if (lengthCertainDir.x < lengthCertainDir.z) {
				cur_pos = getNewPos(unit, cur_pos, lengthCertainDir.z);
				len += lengthCertainDir.z * unitLen;
			}	
			else {
				cur_pos = getNewPos(unit, cur_pos, lengthCertainDir.y);
				len += lengthCertainDir.y * unitLen;
			} 
		}
		else {
			if (lengthCertainDir.y < lengthCertainDir.z) {
				cur_pos = getNewPos(unit, cur_pos, lengthCertainDir.z);
				len += lengthCertainDir.z * unitLen;
			}
			else {
				cur_pos = getNewPos(unit, cur_pos, lengthCertainDir.x);
				len += lengthCertainDir.x * unitLen;
			}
		}

		BasicVoxel currentVoxel = voxelMatrix[int(cur_pos.x)][int(cur_pos.y)][int(cur_pos.z)];
		if (currentVoxel.valid == true) { voxel = currentVoxel; }
	}
	
	return voxel;
}


void main() {
	voxelMatrix[15][15][15] = BasicVoxel(true, vec4(0, 0, 1, 0));
	BasicVoxel test = castRay(ivec2(45, 35), vec3(14, 14, 14), STEP_LIMIT);
	vec4 colTest = vec4(0,0,1,0);
	imageStore(outImage, ivec2(gl_GlobalInvocationID.xy), test.color);
}