# version 430
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable

# define MAX 100
# define PI 3.141592654
# define STEP_LIMIT 2

layout(local_size_x = 16, local_size_y = 16) in;

writeonly uniform image2D outImage;

struct BasicVoxel {
	bool valid;
	vec4 color;
};

BasicVoxel[16][16][16] voxelMatrix;

float roundFloat(float val) { return round(val * 100) / 100; }
float getDistanceNext(float input_float) { return floor(input_float + 1) - input_float; }
vec3 getNewPos(vec3 unit, vec3 old_pos, float selected_length) { 
return vec3(unit.x * selected_length + old_pos.x, unit.y * selected_length + old_pos.y, unit.z * selected_length + old_pos.z); }

float getTimeMaxLen(float timeMax, float unit) { if (unit != 0.0) { return timeMax / unit; } else { return 0.0; } }

BasicVoxel castRay(ivec2 dir, vec3 origin, int step_limit) {
	float alpha = roundFloat(dir.x * PI / 180);
	float beta = roundFloat(dir.y * PI / 180);
	vec3 unit = vec3(roundFloat(cos(alpha) * cos(beta)), roundFloat(sin(beta)), roundFloat(sin(alpha) * cos(beta)));
	
	float len = 0;
	vec3 curPos = origin;
	BasicVoxel voxel = BasicVoxel(false, vec4(0.0, 0.0, 0.0, 0.0));
	BasicVoxel[10] test_list;
	vec4 col = vec4(0, 0, 0, 0);
	
	for(int iteration = 0; iteration < step_limit; iteration += 1) {
		vec3 distNextVox = vec3(getDistanceNext(curPos.x), getDistanceNext(curPos.y), getDistanceNext(curPos.z));
		vec3 lengthCertainDir = vec3(getTimeMaxLen(distNextVox.x, unit.x), getTimeMaxLen(distNextVox.y, unit.y), getTimeMaxLen(distNextVox.z, unit.z));

		if (lengthCertainDir.x < lengthCertainDir.y) {
			if (lengthCertainDir.x < lengthCertainDir.z) {
				curPos = getNewPos(unit, curPos, lengthCertainDir.x);
				len += lengthCertainDir.x;
				
				
			}	
			else {
				curPos = getNewPos(unit, curPos, lengthCertainDir.z);
				len += lengthCertainDir.z;
				
				
			} 
		}
		else {
			if (lengthCertainDir.y < lengthCertainDir.z) {
				curPos = getNewPos(unit, curPos, lengthCertainDir.y);
				len += lengthCertainDir.y;
				
				
			}
			else {
				curPos = getNewPos(unit, curPos, lengthCertainDir.z);
				len += lengthCertainDir.z;
				
				
			}
		}
		
		if (curPos.x < 16) {
			col = vec4(curPos.x / 10, curPos.y / 10, curPos.z / 10, 0);

		}
		
		// BasicVoxel currentVoxel = voxelMatrix[int(curPos.x)][int(curPos.y)][int(curPos.z)];
		// BasicVoxel currentVoxel = voxelMatrix[1][1][1];
		if (currentVoxel.valid == true) { voxel = currentVoxel; }
		
		// voxel = BasicVoxel(true, col);
	}
	
	if (gl_WorkGroupID.x < 10) {
		voxel = test_list[int(gl_WorkGroupID.x)];
	}
	
	return voxel;
}

	
void main() {
	voxelMatrix[1][1][1] = BasicVoxel(true, vec4(0, 1, 0, 0));
	BasicVoxel test = castRay(ivec2(45, 35), vec3(0, 0, 0), STEP_LIMIT);
	vec4 colTest = vec4(0,0,1,0);
	imageStore(outImage, ivec2(gl_GlobalInvocationID.xy), test.color);
}