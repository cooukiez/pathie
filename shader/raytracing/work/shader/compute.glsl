# version 430
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable

# define MAX 100
# define PI 3.141592654
# define STEP_LIMIT 1000
# define ACCURACY 100

layout(local_size_x = 16, local_size_y = 16) in;

writeonly uniform image2D outImage;

struct BasicVoxel {
	bool valid;
	vec4 color;
};

BasicVoxel[16][16][16] voxelMatrix;

float roundFloat(float val) { return round(val * ACCURACY) / ACCURACY; }
float getDistanceNext(float input_float) { return floor(input_float + 1) - input_float; }
float divide(float first, float sec) { if (sec != 0.0) { return first / sec; } else { return first; } }

BasicVoxel castRay(ivec2 dir, vec3 origin, int step_limit) {
	vec4 col = vec4(0, 0, 0, 0);
	float alpha = dir.x * PI / 180;
	float beta = dir.y * PI / 180;
	vec3 unit = vec3(cos(alpha) * cos(beta), sin(beta), sin(alpha) * cos(beta));
	float len = 0;
	vec3 curPos = origin;
	BasicVoxel voxel = BasicVoxel(false, vec4(0.0, 0.0, 0.0, 0.0));
	
	for(int iteration = 0; iteration < step_limit; iteration += 1) {
		vec3 distNextVox = vec3(getDistanceNext(curPos.x), getDistanceNext(curPos.y), getDistanceNext(curPos.z));
		vec3 lengthCertainDir = vec3(divide(distNextVox.x, unit.x), divide(distNextVox.y, unit.y), divide(distNextVox.z, unit.z));

		if (lengthCertainDir.x < lengthCertainDir.y) {
			if (lengthCertainDir.x < lengthCertainDir.z) {
				float selectedLen = lengthCertainDir.x;
				curPos = vec3(curPos.x + distNextVox.y, unit.y * selectedLen + curPos.y, unit.z * selectedLen + curPos.z);
				len += selectedLen;
			}	
			else {
				float selectedLen = lengthCertainDir.z;
				curPos = vec3(unit.x * selectedLen + curPos.x, unit.y * selectedLen + curPos.y, curPos.z + distNextVox.z);
				len += selectedLen;
			} 
		}
		else {
			if (lengthCertainDir.y < lengthCertainDir.z) {
				float selectedLen = lengthCertainDir.y;
				curPos = vec3(unit.x * selectedLen + curPos.x, curPos.y + distNextVox.y, unit.z * selectedLen + curPos.z);
				len += selectedLen;
			}
			else {
				float selectedLen = lengthCertainDir.z;
				curPos = vec3(unit.x * selectedLen + curPos.x, unit.y * selectedLen + curPos.y, curPos.z + distNextVox.z);
				len += selectedLen;
			}
		}
		
		
		// BasicVoxel currentVoxel = voxelMatrix[int(floor(curPos.x))][int(floor(curPos.y))][int(floor(curPos.z))];
		// BasicVoxel currentVoxel = voxelMatrix[1][1][1];
		// if (currentVoxel.valid == true) { voxel = currentVoxel; }
		
		col = vec4(curPos.x / 10, curPos.y / 10, curPos.z / 10, 0);
		
		voxel = BasicVoxel(true, col);
	}
	
	return voxel;
}

	
void main() {
	voxelMatrix[1][1][1] = BasicVoxel(true, vec4(0, 1, 1, 0));
	BasicVoxel test = castRay(ivec2(23, 0), vec3(0, 0, 0), STEP_LIMIT);
	imageStore(outImage, ivec2(gl_GlobalInvocationID.xy), test.color);
}