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

	float rootSpan;
    uint maxRecursion;
	
	uint nodeAtPos;
    uint nodeAtPosRecursion;
    float nodeAtPosSpan;

	float X;
	float Y;
	float Z;
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

uint posToIndex(vec3 pos, float sideLen) {
    return uint(pos.x + pos.y * sqr(sideLen) + pos.z * sideLen);
}

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

void main() {
	vec2 curRes = vec2(uniformBuffer.width, uniformBuffer.height);
	vec2 fragCoord = gl_FragCoord.xy;
	float curTime = float(uniformBuffer.time) / 1000.0 * 0.5;

	fragColor = vec4(0.0);

    vec2 screenPos = (fragCoord * 2.0 - curRes) / curRes.y;
    int curStep;
	
    vec3 rayOrigin = vec3(uniformBuffer.X, uniformBuffer.Y, uniformBuffer.Z);
    vec3 rayDir = normalize(vec3(screenPos, 1.0));

    uint curIndex = uniformBuffer.nodeAtPos; // octreeData[].parent;
    TreeNode curVox = octreeData[curIndex];
    float curVoxSpan = uniformBuffer.nodeAtPosSpan;

    // Position within current Cell / Node
    vec3 localRayOrigin = mod(rayOrigin, curVoxSpan);
    // RayOrigin on the Edge of the Node
    vec3 originOnEdge = rayOrigin - localRayOrigin;
    // ? Used for RayCube Intersection
    vec3 inverseRayDir = 1.0 / max(abs(rayDir), 0.001);
    // ? Mask -> Which Node to choose
    vec3 mask;

    // Should move up one Layer
    bool exitOctree = false;
    // = Depth
    int recursionAmount = int(uniformBuffer.nodeAtPosRecursion);

    // Travelled Distance
    float dist = 0.0;

    vec3 lastMask;
    vec3 normal = vec3(0.0);
    
    // The Octree TraverseLoop
    // Each Iteration either check ...
    // ... If need to go up
    // ... If need to go down
    // ... If hit -> Break
    // ... If Node / Cell is empty -> Go one step forward

    if (fragCoord.x < 1 && fragCoord.y < 1) {
        debugPrintfEXT("\nStart %d", curIndex);
    }

    for (curStep = 0; curStep < uniformBuffer.maxRayLen; curStep += 1) {
        if (dist > uniformBuffer.maxDist) break;

        // Should go up
        if (exitOctree) {
            vec3 newOriginOnEdge = floor(originOnEdge / (curVoxSpan * 2.0)) * (curVoxSpan * 2.0);
            
            localRayOrigin += originOnEdge - newOriginOnEdge;
            originOnEdge = newOriginOnEdge;
            
            // Moving one Layer up -> Decrease RecursionAmount & Double curVoxSpan
            recursionAmount -= 1;
            curVoxSpan *= 2.0;
            
            curIndex = curVox.parent;
            curVox = octreeData[curIndex];

            if (fragCoord.x < 1 && fragCoord.y < 1) {
                debugPrintfEXT("\nUp %d", curIndex);
            }

            // ?
            exitOctree = (recursionAmount > 0) && (abs(dot(mod(originOnEdge / curVoxSpan + 0.5, 2.0) - 1.0 + mask * sign(rayDir) * 0.5, mask)) < 0.1);
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

                curIndex = curVox.children[posToIndex(childMask, 2.0)];
                curVox = octreeData[curIndex];

                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nDown %d", curIndex);
                }

                originOnEdge += childMask * curVoxSpan;
                localRayOrigin -= childMask * curVoxSpan;

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find distance to NearestVoxSurface in direction of Ray
                // No need to call everytime
                vec3 hit = rayCubeIntersect(localRayOrigin, rayDir, inverseRayDir, curVoxSpan);

                mask = vec3(lessThan(hit,min(hit.yzx, hit.zxy)));
                curIndex = octreeData[curVox.parent].children[posToIndex(mask, 2.0)];
                curVox = octreeData[curIndex];
                float len = dot(hit, mask);

                // Moving forward in direciton of Ray
                dist += len;

                // ?
                localRayOrigin += rayDir * len - mask * sign(rayDir) * curVoxSpan;
                vec3 newOriginOnEdge = originOnEdge + mask * sign(rayDir) * curVoxSpan;

                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nForward %d", curIndex);
                }

                // ? Check if need to move up
                exitOctree = (floor(newOriginOnEdge / curVoxSpan * 0.5 + 0.25) != floor(originOnEdge / curVoxSpan * 0.5 + 0.25)) && (recursionAmount > 0);

                originOnEdge = newOriginOnEdge;
                lastMask = mask;
            } else if (state == 2) {
                if (fragCoord.x < 1 && fragCoord.y < 1) {
                    debugPrintfEXT("\nHit %d", curIndex);
                }

                break;
            }
        }
    }

    if (fragCoord.x < 1 && fragCoord.y < 1) {
        debugPrintfEXT("\nFinished %d", curIndex);
        debugPrintfEXT("\n");
    }

    fragColor = vec4(0, 1 - dist / uniformBuffer.maxDist, 0, 0);
}
