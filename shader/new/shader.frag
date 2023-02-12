# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

# define internMaxRecursion 32
# define rot(spin) mat2(cos(spin), sin(spin), - sin(spin), cos(spin))
# define dir(rot) vec3(cos(rot.x) * cos(rot.y), sin(rot.y), sin(rot.x) * cos(rot.y))
# define rad(degree) vec2(3.14 * degree / 180.0)

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
} uniformBuffer;

struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    uint nodeType;
	uint parent;

	uint children[8];

    float baseColor[3];
};

layout (set = 1, binding = 0) readonly buffer OctreeData {
	TreeNode octreeData[2000000];
};

# define detail 1.0
# define sqr(number) (number * number)

// RayCube Intersection on inside of Cube
vec3 rayCubeIntersect(vec3 rayOrigin, vec3 rayDir, vec3 inverseRayDir, float curSpan) {
    return - (sign(rayDir) * (rayOrigin - curSpan * 0.5) - curSpan * 0.5) * inverseRayDir;
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
    vec2 curRot = vec2(uniformBuffer.rotHorizontal, uniformBuffer.rotVertical);
	vec2 fragCoord = gl_FragCoord.xy;
	float curTime = float(uniformBuffer.time) / 1000.0 * 0.5;

	fragColor = vec4(0.0);

    vec2 screenPos = (fragCoord * 2.0 - curRes) / curRes.y;
    int curStep;
	
    vec3 rayOrigin = vec3(uniformBuffer.X, uniformBuffer.Y, uniformBuffer.Z);
    vec3 rayDir = normalize(vec3(screenPos, 1.0));

    vec2 first = vec2(2, 0) / curRes.y;
    vec2 sec = vec2(- 2, 0) / curRes.y;
    vec3 f = normalize(vec3(first, 1.0));
    vec3 s = normalize(vec3(sec, 1.0));

    // if (fragCoord.x > curRes.x - 2.0 && fragCoord.y > curRes.y - 2.0) {
        // debugPrintfEXT("\n%v3f %v3f", firstTest, secTest);
        // debugPrintfEXT("\n%f", tReq);
    // }

    // dir(rad(vec2(30, 30)))

    float offset = 3.14 * 0.5;

    rayDir.yz *= rot(curRot.y / curRes.y * 3.14 - offset);
    rayDir.xz *= rot(curRot.x / curRes.x * 3.14 - offset);

    uint curIndex = 0; // octreeData[].parent;
    TreeNode curNode = octreeData[curIndex];
    float curSpan = uniformBuffer.rootSpan;

    // Position within current Cell / Node
    vec3 localRayOrigin = mod(rayOrigin, curSpan);
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

    for (curStep = 0; curStep < uniformBuffer.maxRayLen; curStep += 1) {
        if (dist > uniformBuffer.maxDist) break;

        // Should go up
        if (exitOctree) {
            if (curNode.parent == 0) { break; }

            vec3 newOriginOnEdge = floor(originOnEdge / (curSpan * 2.0)) * (curSpan * 2.0);
            
            localRayOrigin += originOnEdge - newOriginOnEdge;
            originOnEdge = newOriginOnEdge;
            
            // Moving one Layer upward -> Decrease RecursionAmount & Double curSpan
            recursionAmount -= 1;
            curSpan *= 2.0;
            
            TreeNode parentOfParent = octreeData[octreeData[curNode.parent].parent];
            maskInParentList[recursionAmount] = addDirToMask(maskInParentList[recursionAmount], dirMask);

            curIndex = parentOfParent.children[maskToIndex(maskInParentList[recursionAmount])];
            curNode = octreeData[curIndex];

            exitOctree = (abs(dot(mod((originOnEdge + 0.25) / curSpan + 0.5, 2.0) - 1.0 + dirMask * sign(rayDir) * 0.5, dirMask)) < 0.1);
        } else {
            // Getting Node Type
            uint state = curNode.nodeType;

            // If State == Subdivide && too much Detail -> State = Empty
            if (state == 1 && recursionAmount > uniformBuffer.maxRecursion) { state = 0; }
            if (state == 1 && (curSpan * 0.3) < (abs(s.x * dist - f.x * dist))) { fragColor = vec4(1); break; }

            // If State = Subdivide && no Limit of Detail reached -> Select Child
            if (state == 1 && recursionAmount <= uniformBuffer.maxRecursion) {
                // Moving one Layer down -> Increase RecursionAmount & Half curSpan
                recursionAmount += 1;
                curSpan *= 0.5;

                // Select specific Child
                vec3 childMask = step(vec3(curSpan), localRayOrigin);

                originOnEdge += childMask * curSpan;
                localRayOrigin -= childMask * curSpan;

                curIndex = curNode.children[maskToIndex(childMask)];
                curNode = octreeData[curIndex];
                
                maskInParentList[recursionAmount] = childMask;

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find distance to NearestVoxSurface in direction of Ray
                // No need to call everytime
                vec3 hit = rayCubeIntersect(localRayOrigin, rayDir, inverseRayDir, curSpan);

                dirMask = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));

                float len = dot(hit, dirMask);

                // Moving forward in direciton of Ray
                dist += len;

                localRayOrigin += rayDir * len - dirMask * sign(rayDir) * curSpan;
                vec3 newOriginOnEdge = originOnEdge + dirMask * sign(rayDir) * curSpan;

                maskInParentList[recursionAmount] = addDirToMask(maskInParentList[recursionAmount], dirMask);
                curIndex = octreeData[curNode.parent].children[maskToIndex(maskInParentList[recursionAmount])];
                curNode = octreeData[curIndex];

                exitOctree = (floor(newOriginOnEdge / curSpan * 0.5 + 0.25) != floor(originOnEdge / curSpan * 0.5 + 0.25));

                originOnEdge = newOriginOnEdge;
                lastDirMask = dirMask;
            } else if (state == 2) {
                fragColor = vec4(curNode.baseColor[0], curNode.baseColor[1], curNode.baseColor[2], 0);
                break;
            }
        }
    }    
}
