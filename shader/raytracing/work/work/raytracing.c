# include <stdio.h>
# include <stdbool.h>
# include <math.h>

# define MAX 100

typedef struct vec3 {
    float x, y, z;
} vec3;
typedef struct vec4 {
    float x, y, z, a;
} vec4;
typedef struct ivec3 {
    int x, y, z;
} ivec3;
typedef struct ivec2 {
    int x, y;
} ivec2;

typedef struct BasicVoxel {
	bool valid;
	vec4 color;
} BasicVoxel;

BasicVoxel voxelMatrix[16][16][16];
vec4 color;

void printFloat(float value) {
	char buf[MAX];
    gcvt(value, 6, buf);
    printf("Value is -> %s\n", buf);
}

float getDistanceNext(float input_float) { return floor(input_float + 1) - input_float; }

BasicVoxel castRay(ivec2 dir, vec3 origin) {
	vec3 unit = { cos(dir.x), sin(dir.x), 0.0 };
	bool hit = false;
	float len = 0;
	vec3 cur_pos = origin;
	BasicVoxel voxel = { false, (vec4) { 0, 0, 0, 0 } };
	color = (vec4) { 0.0, 50.0, 0.0, 0.0 };
	while (!hit) {
		vec3 timeMax = { getDistanceNext(cur_pos.x), getDistanceNext(cur_pos.y), 0.0 };
		vec3 timeMaxLength = { timeMax.x / unit.x, timeMax.y / unit.y, 0.0 };
		if (timeMaxLength.x > timeMaxLength.y) {
			cur_pos = (vec3) { unit.x * timeMaxLength.x, unit.y * timeMaxLength.x, 0.0 };
			len += timeMaxLength.x;
		}
		if (timeMaxLength.x < timeMaxLength.y) {
			cur_pos = (vec3) { unit.x * timeMaxLength.y, unit.y * timeMaxLength.y, 0.0 };
			len += timeMaxLength.y;
		}
		if (len > 100) { hit = true; }
		BasicVoxel voxel_at_pos = voxelMatrix[(int)(cur_pos.x)][(int)(cur_pos.y)][(int)(cur_pos.z)];
		if (voxel_at_pos.valid == true) { hit = true; voxel = voxel_at_pos; }
	}
	return voxel;
}

int main() {
    printf("Starting ... \n");
	voxelMatrix[0][5][0] = (BasicVoxel) { true, (vec4) { 100, 0, 0, 0 } };
	BasicVoxel test = castRay((ivec2) { 0, 0 }, (vec3) { 0.0, 0.0, 0.0 });
	printFloat(test.color.x);
    return 0;
}