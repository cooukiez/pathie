# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

layout (set = 0, binding = 0) uniform Uniform {
    uint time;

	float width;
	float height;

	float rawfieldOfView;
	uint maxRayLen;
    float maxDist;

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
};

# define detail 5

# define emptycells 0.5
# define subdivisions 0.95 //should be higher than emptycells

# define sqr(number) (number * number)

float rnd(vec4 v) { return fract(4e4*sin(dot(v,vec4(13.46,41.74,-73.36,14.24))+17.34)); }

//0 is empty, 1 is subdivide and 2 is full
int getvoxel(vec3 p, float size) {
    if (p.x==0.0&&p.y==0.0) {
        return 0;
    }
    
    float val = rnd(vec4(p,size));
    
    if (val < emptycells) {
        return 0;
    } else if (val < subdivisions) {
        return 1;
    } else {
        return 2;
    }
    
    return int(val*val*3.0);
}

//ray-cube intersection, on the inside of the cube
vec3 voxel(vec3 rayOrigin, vec3 rayDir, vec3 ird, float size) {
    return - (sign(rayDir) * (rayOrigin - size * 0.5) - size * 0.5) * ird;;
}

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

void main() {
	vec2 curRes = vec2(uniformBuffer.width, uniformBuffer.height);
	vec2 fragCoord = gl_FragCoord.xy;
	float curTime = float(uniformBuffer.time) / 1000.0 * 0.5;

	fragColor = vec4(0.0);

    vec2 screenPos = (fragCoord * 2.0 - curRes) / curRes.y;
    float size = 1.0;
    int curStep;
	
    vec3 rayOrigin = vec3(uniformBuffer.X, uniformBuffer.Y, uniformBuffer.Z);
    vec3 rayDir = normalize(vec3(screenPos, 1.0));

    vec3 lro = mod(rayOrigin, size);
    vec3 fro = rayOrigin - lro;
    vec3 ird = 1.0 / max(abs(rayDir), 0.001);
    vec3 mask;

    bool exitOctree = false;
    int recursionAmount = 0;

    float dist = 0.0;
    float fdist = 0.0;

    vec3 lastMask;
    vec3 normal = vec3(0.0);
    
    //the octree traverser loop
    //each iteration i either:
    // - check if i need to go up a level
    // - check if i need to go down a level
    // - check if i hit a cube
    // - go one step forward if octree cell is empty
    // - repeat if i did not hit a cube

    for (curStep = 0; curStep < uniformBuffer.maxRayLen; curStep += 1) {
        if (dist > uniformBuffer.maxDist) break;
        if (fragCoord.x < 1 && fragCoord.y < 1) {
            // debugPrintfEXT("\nlro %v3f | fro %v3f | ird %v3f | ro %v3f | rd %v3f", lro, fro, ird, rayOrigin, rayDir);
        }

        //i go up a level
        if (exitOctree) {  
            vec3 newfro = floor(fro / (size * 2.0)) * (size * 2.0);
            
            lro += fro - newfro;
            fro = newfro;
            
            recursionAmount -= 1;
            size *= 2.0;
            
            exitOctree = (recursionAmount > 0) && (abs(dot(mod(fro / size + 0.5, 2.0) - 1.0 + mask * sign(rayDir) * 0.5, mask)) < 0.1);
        }
        else
        {
            //checking what type of cell it is: empty, full or subdivide
            int voxelstate = getvoxel(fro,size);
            if (voxelstate == 1 && recursionAmount > detail)
            {
                voxelstate = 0;
            }
            
            if(voxelstate == 1 && recursionAmount <= detail)
            {
                //if(recursions>detail) break;

                recursionAmount += 1;
                size *= 0.5;

                //find which of the 8 voxels i will enter
                vec3 mask2 = step(vec3(size), lro);
                fro += mask2 * size;
                lro -= mask2 * size;
            }
            //move forward
            else if (voxelstate == 0||voxelstate == 2)
            {
                //raycast and find distance to nearest voxel surface in ray direction
                //i don't need to use voxel() every time, but i do anyway
                vec3 hit = voxel(lro, rayDir, ird, size);

                mask = vec3(lessThan(hit,min(hit.yzx, hit.zxy)));
                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\n%v3f", mask);
                }
                float len = dot(hit, mask);

				if (voxelstate == 2) {
                    break;
                }

                //moving forward in ray direction, and checking if i need to go up a level
                dist += len;
                fdist += len;
                lro += rayDir * len - mask * sign(rayDir) * size;
                vec3 newfro = fro + mask * sign(rayDir) * size;
                exitOctree = (floor(newfro / size * 0.5 + 0.25) != floor(fro / size * 0.5 + 0.25)) && (recursionAmount > 0);
                fro = newfro;
                lastMask = mask;
            }
        }
    }
    
    rayOrigin += rayDir * dist;
    if(curStep < uniformBuffer.maxRayLen && dist < uniformBuffer.maxDist) {
    	float val = fract(dot(fro, vec3(15.23, 754.345, 3.454)));
        vec3 color = sin(val * vec3(39.896,57.3225,48.25)) * 0.5 + 0.5;
    	fragColor = vec4(color * (normal * 0.25 + 0.75), 1.0);
    }

    fragColor = sqrt(fragColor);
}
