# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

# define maxDepth 15
# define maxDistance 4096.0
# define maxSearchDepth 4096

# define sqr(number) (number * number)
# define rot(spin) mat2(cos(spin), sin(spin), - sin(spin), cos(spin))
# define dir(rot) vec3(cos(rot.x) * cos(rot.y), sin(rot.y), sin(rot.x) * cos(rot.y))
# define rad(degree) vec2(3.14 * degree / 180.0)

struct Ray {
    vec3 origin;
    vec3 dir;
};

struct NodeInfo {
    uint index;
    float span;
    int depth;
    vec3 pos;
};

struct Intersection {
    bool intersect;
    float dist;

    NodeInfo info;
};

struct TraverseProp {
    uint locMaxDepth;
    float locMaxDistance;
    uint locMaxSearchDepth;
};

layout (set = 0, binding = 0) uniform Uniform {
    uint time;

    // CurrentResolution
	float width;
	float height;

    // MousePosition
	float MX;
	float MY;

	float rootSpan;

    // PlayerPosition
	float X;
	float Y;
	float Z;
} uniformBuffer;

struct TreeNode {
    // 0 = empty | 1 = subdivide | 2 = full
    uint nodeType;
	uint parent;

	uint children[8];
    float baseColor[3]; // ToDo -> Add transparency
};

layout (set = 1, binding = 0) readonly buffer OctreeData { TreeNode octreeData[2000000]; };
layout (set = 2, binding = 0) readonly buffer LightData { NodeInfo lightData[2000]; };

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

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

Intersection traverseRay(Ray ray, TraverseProp prop) {
    uint curIndex = 0;
    float curSpan = uniformBuffer.rootSpan;

    int curDepth = 0;
    vec3 maskInParentList[maxDepth];

    vec3 localPos = mod(ray.origin, curSpan); // Position within current Cell / Node
    vec3 posOnEdge = ray.origin - localPos; // RayOrigin on the Edge of the Node

    vec3 invRayDir = 1.0 / max(abs(ray.dir), 0.001); // Used for RayCube Intersection

    float dist = 0.0;
    bool intersect = false;

    bool exitOctree = false; // Should move upward
    int curStep;
    vec3 dirMask;

    TreeNode curNode = octreeData[curIndex];
    
    // The Octree TraverseLoop
    // Each Iteration either check ...
    // ... If need to go up
    // ... If need to go down
    // ... If hit -> Break
    // ... If Node / Cell is empty -> Go one step forward

    for (curStep = 0; curStep < prop.locMaxSearchDepth; curStep += 1) {
        if (dist > prop.locMaxDistance) break;

        // Should go up
        if (exitOctree) {
            if (curNode.parent == 0) break;

            vec3 newPosOnEdge = floor(posOnEdge / (curSpan * 2.0)) * (curSpan * 2.0);
            
            localPos += posOnEdge - newPosOnEdge;
            posOnEdge = newPosOnEdge;
            
            // Moving one Layer upward -> Decrease RecursionAmount & Double curSpan
            curDepth -= 1;
            curSpan *= 2.0;
            
            TreeNode parentOfParent = octreeData[octreeData[curNode.parent].parent];
            maskInParentList[curDepth] = addDirToMask(maskInParentList[curDepth], dirMask);

            curIndex = parentOfParent.children[maskToIndex(maskInParentList[curDepth])];
            curNode = octreeData[curIndex];

            exitOctree = (abs(dot(mod((posOnEdge + 0.25) / curSpan + 0.5, 2.0) - 1.0 + dirMask * sign(ray.dir) * 0.5, dirMask)) < 0.1);
        } else {
            // Getting Node Type
            uint state = curNode.nodeType;

            // If State == Subdivide && too much Detail -> State = Empty
            if (state == 1 && curDepth > prop.locMaxDepth) state = 2;

            // If State = Subdivide && no Limit of Detail reached -> Select Child
            if (state == 1) {
                // Moving one Layer down -> Increase RecursionAmount & Half curSpan
                curDepth += 1;
                curSpan *= 0.5;

                // Select specific Child
                vec3 childMask = step(vec3(curSpan), localPos);

                posOnEdge += childMask * curSpan;
                localPos -= childMask * curSpan;

                curIndex = curNode.children[maskToIndex(childMask)];
                curNode = octreeData[curIndex];
                
                maskInParentList[curDepth] = childMask;

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find distance to NearestVoxSurface in direction of Ray
                // No need to call everytime
                vec3 hit = rayCubeIntersect(localPos, ray.dir, invRayDir, curSpan);

                dirMask = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));

                float len = dot(hit, dirMask);

                // Moving forward in direciton of Ray
                dist += len;

                localPos += ray.dir * len - dirMask * sign(ray.dir) * curSpan;
                vec3 newPosOnEdge = posOnEdge + dirMask * sign(ray.dir) * curSpan;

                maskInParentList[curDepth] = addDirToMask(maskInParentList[curDepth], dirMask);
                curIndex = octreeData[curNode.parent].children[maskToIndex(maskInParentList[curDepth])];
                curNode = octreeData[curIndex];

                exitOctree = (floor(newPosOnEdge / curSpan * 0.5 + 0.25) != floor(posOnEdge / curSpan * 0.5 + 0.25));

                posOnEdge = newPosOnEdge;
            } else if (state > 1) {
                intersect = true;
                break;
            }
        }
    }

    NodeInfo info = NodeInfo(curIndex, curSpan, curDepth, posOnEdge + localPos);
    return Intersection(intersect, dist, info);
}

Intersection traversePrimaryRay(vec2 coord, vec2 res, vec2 mouse) {
    vec2 screenPos = (coord * 2.0 - res) / res.y;

    vec3 rayOrigin = vec3(uniformBuffer.X, uniformBuffer.Y, uniformBuffer.Z);
    vec3 rayDir = normalize(vec3(screenPos, 1.0));

    float offset = 3.14 * 0.5;
    rayDir.yz *= rot(mouse.y / res.y * 3.14 - offset);
    rayDir.xz *= rot(mouse.x / res.x * 3.14 - offset);

    Ray ray = Ray(rayOrigin, rayDir);
    TraverseProp prop = TraverseProp(maxDepth, maxDistance, maxSearchDepth);

    return traverseRay(ray, prop);
}

uint selectLightSource(vec3 pos) {
    return 1;
}

void main() {
    fragColor = vec4(0.0);

    vec2 coord = gl_FragCoord.xy;
	vec2 res = vec2(uniformBuffer.width, uniformBuffer.height);
    vec2 mouse = vec2(uniformBuffer.MX, uniformBuffer.MY);
	
	float time = float(uniformBuffer.time) / 1000.0 * 0.5;

    // if (coord.x < 1 && coord.y < 1) {
        // debugPrintfEXT("");
    // }

    // dir(rad(vec2(30, 30)))
    
    Intersection intSec = traversePrimaryRay(coord, res, mouse);
    TreeNode node = octreeData[intSec.info.index];
    vec3 color = vec3(node.baseColor[0], node.baseColor[1], node.baseColor[2]);
    if (intSec.intersect) fragColor = vec4(color, 1);
}
