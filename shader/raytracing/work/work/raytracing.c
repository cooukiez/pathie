# include <stdio.h>
# include <stdbool.h>
# include <math.h>

# define MAX 100
# define PI 3.141592654

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

float roundFloat(float val) { return roundf(val * 1000) / 1000; }
float getDistanceNext(float input_float) { return floor(input_float + 1) - input_float; }
float getTimeMaxLen(float timeMax, float unit) { if (unit != 0.0) { return timeMax / unit; } else { return 0.0; } }

BasicVoxel castRay(ivec2 dir, vec3 origin) {
	vec3 unit = { roundFloat(cos(roundFloat(dir.x * PI / 180))), roundFloat(sin(roundFloat(dir.x * PI / 180))), 0.0 };
	float unitLen = sqrt(roundFloat(unit.x * unit.x + unit.y * unit.y));
	bool hit = false;
	float len = 0;
	vec3 cur_pos = origin;
	BasicVoxel voxel = { false, (vec4) { 0.0, 0.0, 0.0, 0.0 } };
	color = (vec4) { 0.0, 50.0, 0.0, 0.0 };
	while (len < 100 && hit == false) {
		vec3 timeMax = { getDistanceNext(cur_pos.x), getDistanceNext(cur_pos.y), 0.0 };
		vec3 timeMaxLength = { getTimeMaxLen(timeMax.x, unit.x), getTimeMaxLen(timeMax.y, unit.y), 0.0 };
		if (timeMaxLength.x < timeMaxLength.y) {
			cur_pos = (vec3) { unit.x * timeMaxLength.y + cur_pos.x, unit.y * timeMaxLength.y + cur_pos.y, 0.0 };
			len += timeMaxLength.y * unitLen;
		}
		else {
			cur_pos = (vec3) { unit.x * timeMaxLength.x + cur_pos.x, unit.y * timeMaxLength.x  + cur_pos.y, 0.0 };
			len += timeMaxLength.x * unitLen;
		}

		int pos_y = (int)(cur_pos.y);
		BasicVoxel voxel_at_pos = voxelMatrix[(int)(cur_pos.x)][(int)(cur_pos.y)][(int)(cur_pos.z)];
		if (voxel_at_pos.valid == true) { hit = true; voxel = voxel_at_pos; }
	}
	return voxel;
}

int main() {
    printf("Starting ... \n");
	voxelMatrix[5][5][0] = (BasicVoxel) { true, (vec4) { 100, 0, 0, 0 } };
	BasicVoxel test = castRay((ivec2) { 45, 0 }, (vec3) { 0.0, 0.0, 0.0 });
	printf("Result\n");
	printFloat(test.color.x);
    return 0;
}