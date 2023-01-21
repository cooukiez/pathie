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

# define detail 10

# define emptycells 0.5
# define subdivisions 0.99 //should be higher than emptycells

# define sqr(number) (number * number)

float rnd(vec4 v) { return fract(4e4*sin(dot(v,vec4(13.46,41.74,-73.36,14.24))+17.34)); }

//0 is empty, 1 is subdivide and 2 is full
int getvoxel(vec3 position, float size) {
    if (position.x == 0.0 && position.y == 0.0) {
        return 0;
    }
    
    float val = rnd(vec4(position,size));
    
    if (val < emptycells) {
        return 0;
    } else if (val < subdivisions) {
        return 1;
    } else {
        return 2;
    }
    
    return int(val * val * 3.0);
}

//ray-cube intersection, on the inside of the cube
vec3 voxel(vec3 rayOrigin, vec3 rayDir, vec3 inverseRayDir, float size) {
    return - (sign(rayDir) * (rayOrigin - size * 0.5) - size * 0.5) * inverseRayDir;;
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

    // Position within current Cell / Node
    vec3 localRayOrigin = mod(rayOrigin, size);
    // RayOrigin on the Edge of the Node
    vec3 originOnEdge = rayOrigin - localRayOrigin;
    // ?
    vec3 inverseRayDir = 1.0 / max(abs(rayDir), 0.001);
    // ? Mask -> Which Node to choose
    vec3 mask;

    // Should move up one Layer
    bool exitOctree = false;
    // = Depth
    int recursionAmount = 0;

    // Travelled Distance
    float dist = 0.0;
    // ? -> Replace Name
    float fdist = 0.0;

    vec3 lastMask;
    vec3 normal = vec3(0.0);
    
    // The Octree TraverseLoop
    // Each Iteration either check ...
    // ... If need to go up
    // ... If need to go down
    // ... If hit -> Break
    // ... If Node / Cell is empty -> Go one step forward

    for (curStep = 0; curStep < uniformBuffer.maxRayLen; curStep += 1) {
        if (dist > uniformBuffer.maxDist) break;
        if (fragCoord.x < 1 && fragCoord.y < 1) {
            // debugPrintfEXT("\nlro %v3f | originOnEdge %v3f | inverseRayDir %v3f | ro %v3f | rd %v3f", localRayOrigin, originOnEdge, inverseRayDir, rayOrigin, rayDir);
        }

        // Should go up
        if (exitOctree) {
            // ?
            vec3 newOriginOnEdge = floor(originOnEdge / (size * 2.0)) * (size * 2.0);
            
            localRayOrigin += originOnEdge - newOriginOnEdge;
            originOnEdge = newOriginOnEdge;
            
            // Moving one Layer up -> Decrease RecursionAmount & Double Size
            recursionAmount -= 1;
            size *= 2.0;
            
            // ?
            exitOctree = (recursionAmount > 0) && (abs(dot(mod(originOnEdge / size + 0.5, 2.0) - 1.0 + mask * sign(rayDir) * 0.5, mask)) < 0.1);
        }
        else
        {
            // Getting Node Type
            int state = getvoxel(originOnEdge, size); // Replace

            // If State == Subdivide && too much Detail -> State = Empty
            if (state == 1 && recursionAmount > detail) { state = 0; }
            
            // If State = Subdivide && no Limit of Detail reached -> Select Child
            if(state == 1 && recursionAmount <= detail) {
                // Moving one Layer down -> Increase RecursionAmount & Half Size
                recursionAmount += 1;
                size *= 0.5;

                // Select specific Child
                vec3 childMask = step(vec3(size), localRayOrigin);

                originOnEdge += childMask * size;
                localRayOrigin -= childMask * size;

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find distance to NearestVoxSurface in direction of Ray
                // No need to call everytime
                vec3 hit = voxel(localRayOrigin, rayDir, inverseRayDir, size);

                mask = vec3(lessThan(hit,min(hit.yzx, hit.zxy)));
                float len = dot(hit, mask);

                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    // debugPrintfEXT("\n%v3f", mask);
                }

                // Moving forward in direciton of Ray
                dist += len;
                fdist += len;

                // ?
                localRayOrigin += rayDir * len - mask * sign(rayDir) * size;
                vec3 newOriginOnEdge = originOnEdge + mask * sign(rayDir) * size;

                // ? Check if need to move up
                exitOctree = (floor(newOriginOnEdge / size * 0.5 + 0.25) != floor(originOnEdge / size * 0.5 + 0.25)) && (recursionAmount > 0);

                originOnEdge = newOriginOnEdge;
                lastMask = mask;
            } else if (state == 2) { break; }
        }
    }
    
    // ?
    rayOrigin += rayDir * dist;
    if(curStep < uniformBuffer.maxRayLen && dist < uniformBuffer.maxDist) {
        // ?
    	float val = fract(dot(originOnEdge, vec3(15.23, 754.345, 3.454)));
        vec3 color = sin(val * vec3(39.896,57.3225,48.25)) * 0.5 + 0.5;

        // ?
    	fragColor = vec4(color * (normal * 0.25 + 0.75), 1.0);
    }

    // ?
    fragColor = sqrt(fragColor);
}
