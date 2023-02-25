# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable
# extension GL_EXT_scalar_block_layout : enable

# define maxDepth 17
# define maxDistance 4096.0
# define maxSearchDepth 4096

# define sqr(number) (number * number)
# define rot(spin) mat2(cos(spin), sin(spin), - sin(spin), cos(spin))
# define dir(rot) vec3(cos(rot.x) * cos(rot.y), sin(rot.y), sin(rot.x) * cos(rot.y))
# define rad(degree) vec2(3.14 * degree / 180.0)

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

layout (std430, set = 0, binding = 0) uniform Uniform {
    vec4 pos;

    vec2 res;
    vec2 mouse;

    float rootSpan;

    uint time;
} uniformBuffer;

struct TreeNode {
    uint children[8];
    
    // 0 = empty | 1 = subdivide | 2 = full
    uint nodeType;
	uint parent;
    vec4 baseColor; // ToDo -> Add transparency
};

layout (std430, set = 1, binding = 0) buffer OctreeData { TreeNode octreeData[40000]; };

struct Ray {
    vec3 origin;
    vec3 dir;
};

// Position in Octree
struct PosInfo {
    vec3 maskInParent[maxDepth]; // Offset of array wrong

    vec3 localPos; // Position within current Cell / Node
    vec3 posOnEdge; // RayOrigin on the Edge of the Node

    uint index;
    float span;
    int depth;
};

layout (set = 2, binding = 0) readonly buffer LightData { PosInfo lightData[2000]; };

struct Intersection {
    bool intersect;
    float dist;
    PosInfo info;
};

struct TraverseProp {
    // Max
    uint depth;
    float dist;
    uint searchDepth;
};

// RayCube Intersection on inside of Cube
vec3 rayCubeIntersect(vec3 origin, vec3 dir, vec3 invDir, float span) {
    return - (sign(dir) * (origin - span * 0.5) - span * 0.5) * invDir;
}

vec3 addDirToMask(vec3 mask, vec3 dirMask) {
    return abs(mask - dirMask);
}

// Simple Hashing Scheme
uint maskToIndex(vec3 mask) {
    return uint(mask.x + mask.y * 4.0 + mask.z * 2.0);
}

Intersection traverseRay(Ray ray, PosInfo info, TraverseProp prop) {
    vec3 maskInParent[maxDepth];
    vec3 invRayDir = 1.0 / max(abs(ray.dir), 0.001); // Used for RayCube Intersection

    bool intersect = false;
    bool exitOctree = false; // Should move upward
    
    vec3 dirMask;
    TreeNode node = octreeData[info.index];

    float dist = 0;
    
    // The Octree TraverseLoop
    // Each Iteration either check ...
    // ... If need to go up
    // ... If need to go down
    // ... If hit -> Break
    // ... If Node / Cell is empty -> Go one step forward

    int curStep;
    for (curStep = 0; curStep < prop.searchDepth; curStep += 1) {
        if (dist > prop.dist) break;

        // Should go up
        if (exitOctree) {
            if (node.parent == 0 || info.depth < 1) break;

            vec3 newPosOnEdge = floor(info.posOnEdge / (info.span * 2.0)) * (info.span * 2.0);
            
            info.localPos += info.posOnEdge - newPosOnEdge;
            info.posOnEdge = newPosOnEdge;
            
            // Moving one Layer upward -> Decrease RecursionAmount & Double info.span
            info.depth -= 1;
            info.span *= 2.0;
            
            TreeNode grandParent = octreeData[octreeData[node.parent].parent];
            vec3 posMask = abs(info.maskInParent[info.depth] - dirMask);
            info.maskInParent[info.depth] = posMask;

            info.index = grandParent.children[maskToIndex(posMask)];
            node = octreeData[info.index];

            exitOctree = (abs(dot(mod((info.posOnEdge + 0.25) / info.span + 0.5, 2.0) - 1.0 + dirMask * sign(ray.dir) * 0.5, dirMask)) < 0.1);
        } else {
            // Getting Node Type
            uint state = node.nodeType;

            // If State == Subdivide && too much Detail -> State = Empty
            if (state == 1 && info.depth > prop.depth) state = 0;

            // If State = Subdivide && no Limit of Detail reached -> Select Child
            if (state == 1) {
                // Moving one Layer down -> Increase RecursionAmount & Half info.span
                info.depth += 1;
                info.span *= 0.5;

                // Select specific Child
                vec3 childMask = step(vec3(info.span), info.localPos);

                info.posOnEdge += childMask * info.span;
                info.localPos -= childMask * info.span;

                info.index = node.children[maskToIndex(childMask)];
                node = octreeData[info.index];
                
                info.maskInParent[info.depth] = childMask;

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find info.distance to NearestNodeSurface in direction of Ray
                // No need to call everytime
                vec3 hit = rayCubeIntersect(info.localPos, ray.dir, invRayDir, info.span);

                dirMask = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));

                float len = dot(hit, dirMask);

                // Moving forward in direciton of Ray
                dist += len;

                info.localPos += ray.dir * len - dirMask * sign(ray.dir) * info.span;
                vec3 newPosOnEdge = info.posOnEdge + dirMask * sign(ray.dir) * info.span;

                vec3 posMask = abs(info.maskInParent[info.depth] - dirMask);
                info.maskInParent[info.depth] = posMask;

                info.index = octreeData[node.parent].children[maskToIndex(posMask)];
                node = octreeData[info.index];

                exitOctree = (floor(newPosOnEdge / info.span * 0.5 + 0.25) != floor(info.posOnEdge / info.span * 0.5 + 0.25));

                info.posOnEdge = newPosOnEdge;
            } else if (state > 1) {
                intersect = true;
                break;
            }
        }
    }

    return Intersection(intersect, dist, info);
}

Intersection traversePrimaryRay(vec2 coord, vec2 res, vec2 mouse) {
    vec2 screenPos = (coord * 2.0 - res) / res.y;

    vec3 origin = uniformBuffer.pos.xyz;
    vec3 dir = normalize(vec3(screenPos, 1.0));

    float offset = 3.14 * 0.5;
    dir.yz *= rot(mouse.y / res.y * 3.14 - offset);
    dir.xz *= rot(mouse.x / res.x * 3.14 - offset);

    Ray ray = Ray(origin, dir);

    vec3 maskInParent[maxDepth];
    vec3 localPos = mod(origin, uniformBuffer.rootSpan);
    vec3 posOnEdge = origin - localPos;
    PosInfo info = PosInfo(maskInParent, localPos, posOnEdge, 0, uniformBuffer.rootSpan * 2.0, 0);

    TraverseProp prop = TraverseProp(maxDepth, maxDistance, maxSearchDepth);

    return traverseRay(ray, info, prop);
}

Intersection genShadowRay(Intersection lastIntSec) {
    NodeInfo light = lightData[0];

    vec3 origin = lastIntSec.info.posOnEdge + lastIntSec.info.localPos;
    vec3 dir = normalize(origin - vec(light.pos));

    // rayOrigin += rayDir * curIntSec.info.span;

    Ray ray = Ray(origin, dir);
    TraverseProp prop = TraverseProp(maxDepth, maxDistance, maxSearchDepth);

    Intersection intSec = traverseRay(ray, prop);
    if (octreeData[intSec.info.index].nodeType != 3) {
        intSec.intersect = false;
    }

    if (gl_FragCoord.x < 1 && gl_FragCoord.y < 1) {
        // debugPrintfEXT("\n%d", intSec.info.index);
    }

    return intSec;
}

void main() {
    fragColor = vec4(0.0);

    vec2 coord = gl_FragCoord.xy;
	vec2 res = uniformBuffer.res;
    vec2 mouse = uniformBuffer.mouse;
	
	float time = float(uniformBuffer.time) / 1000.0 * 0.5;

    // if (gl_FragCoord.x < 1 && gl_FragCoord.y < 1) {
        // debugPrintfEXT("");
    // }

    // dir(rad(vec2(30, 30)))
    
    Intersection intSec = traversePrimaryRay(coord, res, mouse);
    TreeNode node = octreeData[intSec.info.index];
    
    if (intSec.intersect) {
        fragColor = node.baseColor;
        // if (octreeData[intSec.info.index].nodeType == 3) {
        //     fragColor = vec4(color, 1);
        // } else {
        //     Intersection shadowIntSec = genShadowRay(intSec);
        
        //     if (shadowIntSec.intersect) {
        //         fragColor = vec4(shadowIntSec.dist / 100);
        //     } else {
        //         fragColor = vec4(0.3);
        //     }
        // }
    }
}
