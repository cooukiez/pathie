# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

# define internMaxRecursion 32
# define rot(spin) mat2(cos(spin), sin(spin), - sin(spin), cos(spin))

layout (set = 0, binding = 0) uniform Uniform {
    uint time;

	float width;
	float height;

	float rawfieldOfView;
	uint maxRayLen;
    float maxDist;

	float rotHorizontal;
	float rotVertical;

	float rootSpan;
    uint maxRecursion;

	float X;
	float Y;
	float Z;

    float t1;
    float t2;
} uniformBuffer;

struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    uint nodeType;
	uint parent;

	uint children[8];
};

layout (set = 1, binding = 0) readonly buffer OctreeData {
	TreeNode octreeData[2000];
};

# define detail 1.0
# define sqr(number) (number * number)

// RayCube Intersection on inside of Cube
vec3 rayCubeIntersect(vec3 rayOrigin, vec3 rayDir, vec3 inverseRayDir, float curVoxSpan) {
    return - (sign(rayDir) * (rayOrigin - curVoxSpan * 0.5) - curVoxSpan * 0.5) * inverseRayDir;
}

// Simple Hashing Scheme
uint maskToIndex(vec3 mask) {
    return uint(mask.x + mask.y * 4.0 + mask.z * 2.0);
}

vec3 addDirToMask(vec3 mask, vec3 dirMask) {
    return abs(mask - dirMask);
}

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

void main() {
	vec2 curRes = vec2(uniformBuffer.width, uniformBuffer.height);
    vec2 curRot = vec2(uniformBuffer.rotHorizontal, uniformBuffer.rotVertical) * - 1;
	vec2 fragCoord = gl_FragCoord.xy;
	float curTime = float(uniformBuffer.time) / 1000.0 * 0.5;

	fragColor = vec4(0.0);

    vec2 screenPos = (fragCoord * 2.0 - curRes) / curRes.y;
    int curStep;
	
    vec3 rayOrigin = vec3(uniformBuffer.X, uniformBuffer.Y, uniformBuffer.Z);
    vec3 rayDir = normalize(vec3(screenPos, 1.0));

    rayDir.yz *= rot(curRot.y / curRes.y * 3.14 - 3.14 * 0.5);
    rayDir.xz *= rot(curRot.x / curRes.x * 3.14 * 2.0 - 3.14);

    uint curIndex = 0; // octreeData[].parent;
    TreeNode curVox = octreeData[curIndex];
    float curVoxSpan = uniformBuffer.rootSpan;

    // Position within current Cell / Node
    vec3 localRayOrigin = mod(rayOrigin, curVoxSpan);
    // RayOrigin on the Edge of the Node
    vec3 originOnEdge = rayOrigin - localRayOrigin;
    // ? Used for RayCube Intersection
    vec3 inverseRayDir = 1.0 / max(abs(rayDir), 0.001);

    // Should move up one Layer
    bool exitOctree = false;
    // = Depth
    int recursionAmount = 0;

    // Travelled Distance
    float dist = 0.0;

    vec3 dirMask;
    vec3 lastDirMask;

    vec3 maskInParentList[internMaxRecursion];
    
    // The Octree TraverseLoop
    // Each Iteration either check ...
    // ... If need to go up
    // ... If need to go down
    // ... If hit -> Break
    // ... If Node / Cell is empty -> Go one step forward

    if (fragCoord.x < 1 && fragCoord.y < 1) {
        debugPrintfEXT("\nStart %d %v3f %v3f", curIndex, rayDir, sign(rayDir));
    }

    for (curStep = 0; curStep < uniformBuffer.maxRayLen; curStep += 1) {
        if (dist > uniformBuffer.maxDist) break;

        // Should go up
        if (exitOctree) {
            if (curVox.parent == 0) {
                break;
            }

            vec3 newOriginOnEdge = floor(originOnEdge / (curVoxSpan * 2.0)) * (curVoxSpan * 2.0);
            
            localRayOrigin += originOnEdge - newOriginOnEdge;
            originOnEdge = newOriginOnEdge;
            
            // Moving one Layer up -> Decrease RecursionAmount & Double curVoxSpan
            recursionAmount -= 1;
            curVoxSpan *= 2.0;

            if (fragCoord.x < 1 && fragCoord.y < 1) {
                debugPrintfEXT("\ndirMask %v3f PIPM %v3f", dirMask, maskInParentList[recursionAmount]);
            }
            
            TreeNode parentOfParent = octreeData[octreeData[curVox.parent].parent];
            maskInParentList[recursionAmount] = addDirToMask(maskInParentList[recursionAmount], dirMask);

            curIndex = parentOfParent.children[maskToIndex(maskInParentList[recursionAmount])];
            curVox = octreeData[curIndex];

            if (fragCoord.x < 1 && fragCoord.y < 1) {
                vec3 curTestPos = originOnEdge + localRayOrigin;
                debugPrintfEXT("\nUp %d %f %v3f %v3f", curIndex, curVoxSpan, originOnEdge, localRayOrigin);
            }

            exitOctree = (abs(dot(mod((originOnEdge + 0.25) / curVoxSpan + 0.5, 2.0) - 1.0 + dirMask * sign(rayDir) * 0.5, dirMask)) < 0.1) && (recursionAmount > 0);
        } else {
            // Getting Node Type
            uint state = curVox.nodeType;

            // If State == Subdivide && too much Detail -> State = Empty
            if (state == 1 && recursionAmount > uniformBuffer.maxRecursion) { state = 0; }
            
            // If State = Subdivide && no Limit of Detail reached -> Select Child
            if (state == 1 && recursionAmount <= uniformBuffer.maxRecursion) {
                // Moving one Layer down -> Increase RecursionAmount & Half curVoxSpan
                recursionAmount += 1;
                curVoxSpan *= 0.5;

                // Select specific Child
                vec3 childMask = step(vec3(curVoxSpan), localRayOrigin);

                originOnEdge += childMask * curVoxSpan;
                localRayOrigin -= childMask * curVoxSpan;

                curIndex = curVox.children[maskToIndex(childMask)];
                curVox = octreeData[curIndex];
                
                maskInParentList[recursionAmount] = childMask;

                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nDown %d %v3f %v3f", curIndex, originOnEdge, localRayOrigin);
                }

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find distance to NearestVoxSurface in direction of Ray
                // No need to call everytime
                vec3 hit = rayCubeIntersect(localRayOrigin, rayDir, inverseRayDir, curVoxSpan);

                dirMask = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));
                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nmask %v3f", dirMask);
                }
                
                float len = dot(hit, dirMask);

                // Moving forward in direciton of Ray
                dist += len;

                // ?
                localRayOrigin += rayDir * len - dirMask * sign(rayDir) * curVoxSpan;
                vec3 newOriginOnEdge = originOnEdge + dirMask * sign(rayDir) * curVoxSpan;

                maskInParentList[recursionAmount] = addDirToMask(maskInParentList[recursionAmount], dirMask);
                curIndex = octreeData[curVox.parent].children[maskToIndex(maskInParentList[recursionAmount])];
                curVox = octreeData[curIndex];

                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nForward %d %v3f %v3f", curIndex, newOriginOnEdge, localRayOrigin);
                }

                // ? Check if need to move up
                exitOctree = (floor(newOriginOnEdge / curVoxSpan * 0.5 + 0.25) != floor(originOnEdge / curVoxSpan * 0.5 + 0.25)) && (recursionAmount > 0);

                originOnEdge = newOriginOnEdge;
                lastDirMask = dirMask;
            } else if (state == 2) {
                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nHit %d", curIndex);
                }
                
                fragColor = vec4(0, 1 - dist / uniformBuffer.maxDist * 2, 0, 0);
                
                break;
            }
        }
    }

    if (fragCoord.x < 1 && fragCoord.y < 1) {
        debugPrintfEXT("\nFinished %d", curIndex);
        debugPrintfEXT("\n");
    }

    
}
