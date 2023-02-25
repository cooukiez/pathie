# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable
# extension GL_EXT_scalar_block_layout : enable

# define maxDepth 15
# define maxDistance 4096.0
# define maxSearchDepth 4096

# define sqr(number) (number * number)
# define rot(spin) mat2(cos(spin), sin(spin), - sin(spin), cos(spin))
# define dir(rot) vec3(cos(rot.x) * cos(rot.y), sin(rot.y), sin(rot.x) * cos(rot.y))
# define rad(degree) vec2(3.14 * degree / 180.0)

# define f(v) vec3(v)
# define b(v) vec4(v, 0)

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

layout (std430, set = 0, binding = 0) uniform Uniform {
    vec4 pos;
    vec2 mouse;
	vec2 res;
    uint time;
} uniformBuffer;

struct Ray {
    vec4 origin;
    vec4 dir;
};

struct Traverse {
    vec4 maskInParent[maxDepth]; // Offset of array wrong

    Ray ray;
    vec4 localPos; // Position within current Cell / Node
    vec4 posOnEdge; // RayOrigin on the Edge of the Node

    uint index;
    uint parent;
    
    float dist;
    float span;

    int depth;
};

layout (std430, set = 0, binding = 1) uniform TraverseBuffer {
    Traverse traverseBuffer;
};

struct TreeNode {
    vec4 baseColor; // ToDo -> Add transparency
    uint children[8];
    
    // 0 = empty | 1 = subdivide | 2 = full
    uint nodeType;
	uint parent;
};

layout (std430, set = 1, binding = 0) buffer OctreeData { TreeNode octreeData[40000]; };
layout (set = 2, binding = 0) readonly buffer LightData { Traverse lightData[2000]; };

struct TraverseProp {
    uint locMaxDepth;
    float locMaxDistance;
    uint locMaxSearchDepth;
};

struct Intersection {
    bool intersect;
    Traverse traverse;
};

// RayCube Intersection on inside of Cube
vec3 rayCubeIntersect(vec3 rayOrigin, vec3 rayDir, vec3 inverseRayDir, float curSpan) {
    return - (sign(rayDir) * (rayOrigin - curSpan * 0.5) - curSpan * 0.5) * inverseRayDir;
}

// Simple Hashing Scheme
uint maskToIndex(vec3 mask) {
    return uint(mask.x + mask.y * 4.0 + mask.z * 2.0);
}

Intersection traverseRay(inout Traverse trav, TraverseProp prop) {
    Ray ray = trav.ray;
    vec3 invRayDir = 1.0 / max(abs(f(ray.dir)), 0.001); // Used for RayCube Intersection

    bool intersect = false;
    bool exitOctree = false; // Should move upward
    
    vec3 dirMask;
    TreeNode node = octreeData[trav.index];

    
    // if (gl_FragCoord.x < 1 && gl_FragCoord.y < 1) {
        // debugPrintfEXT("");
    // }
    
    // The Octree TraverseLoop
    // Each Iteration either check ...
    // ... If need to go up
    // ... If need to go down
    // ... If hit -> Break
    // ... If Node / Cell is empty -> Go one step forward

    int curStep;
    for (curStep = 0; curStep < prop.locMaxSearchDepth; curStep += 1) {
        if (trav.dist > prop.locMaxDistance) break;

        // Should go up
        if (exitOctree) {
            if (trav.parent == 0) break;

            vec3 newPosOnEdge = floor(f(trav.posOnEdge) / (trav.span * 2.0)) * (trav.span * 2.0);
            
            trav.localPos += b(f(trav.posOnEdge) - newPosOnEdge);
            trav.posOnEdge = b(newPosOnEdge);
            
            // Moving one Layer upward -> Decrease RecursionAmount & Double trav.span
            trav.depth -= 1;
            trav.span *= 2.0;
            
            TreeNode grandParent = octreeData[octreeData[trav.parent].parent];
            vec3 posMask = abs(f(trav.maskInParent[trav.depth]) - dirMask);

            trav.maskInParent[trav.depth] = ivec4(posMask, 0);
            trav.index = grandParent.children[maskToIndex(posMask)];
            node = octreeData[trav.index];

            exitOctree = (abs(dot(mod((f(trav.posOnEdge) + 0.25) / trav.span + 0.5, 2.0) - 1.0 + dirMask * sign(f(ray.dir)) * 0.5, dirMask)) < 0.1);
        } else {
            // Getting Node Type
            uint state = node.nodeType;

            // If State == Subdivide && too much Detail -> State = Empty
            if (state == 1 && trav.depth > prop.locMaxDepth) state = 2;

            // If State = Subdivide && no Limit of Detail reached -> Select Child
            if (state == 1) {
                // Moving one Layer down -> Increase RecursionAmount & Half trav.span
                trav.depth += 1;
                trav.span *= 0.5;

                // Select specific Child
                vec3 childMask = step(vec3(trav.span), f(trav.localPos));

                trav.posOnEdge += b(childMask * trav.span);
                trav.localPos -= b(childMask * trav.span);

                trav.index = node.children[maskToIndex(childMask)];
                node = octreeData[trav.index];
                
                trav.maskInParent[trav.depth] = ivec4(childMask, 0);

            // Move forward or stop -> 0 = Empty , 2 = Full
            } else if (state == 0) {
                // Raycast and find distance to NearestVoxSurface in direction of Ray
                // No need to call everytime
                vec3 hit = rayCubeIntersect(f(trav.localPos), f(ray.dir), invRayDir, trav.span);

                dirMask = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));

                float len = dot(hit, dirMask);

                // Moving forward in direciton of Ray
                trav.dist += len;

                trav.localPos += b(f(ray.dir) * len - dirMask * sign(f(ray.dir)) * trav.span);
                vec3 newPosOnEdge = f(trav.posOnEdge) + dirMask * sign(f(ray.dir)) * trav.span;

                vec3 posMask = abs(f(trav.maskInParent[trav.depth]) - dirMask);

                trav.maskInParent[trav.depth] = ivec4(posMask, 0);
                trav.index = octreeData[trav.parent].children[maskToIndex(posMask)];
                node = octreeData[trav.index];

                exitOctree = (floor(newPosOnEdge / trav.span * 0.5 + 0.25) != floor(f(trav.posOnEdge) / trav.span * 0.5 + 0.25));

                trav.posOnEdge = b(newPosOnEdge);
            } else if (state > 1) {
                intersect = true;
                break;
            }
        }
    }

    return Intersection(intersect, trav);
}

Intersection traversePrimaryRay(vec2 coord, vec2 res, vec2 mouse) {
    vec4 maskInParent[maxDepth];
    vec2 screenPos = (coord * 2.0 - res) / res.y;

    vec3 rayOrigin = uniformBuffer.pos.xyz;
    vec3 rayDir = normalize(vec3(screenPos, 1.0));

    float offset = 3.14 * 0.5;
    rayDir.yz *= rot(mouse.y / res.y * 3.14 - offset);
    rayDir.xz *= rot(mouse.x / res.x * 3.14 - offset);

    Traverse trav = traverseBuffer;
    trav.ray = Ray(b(rayOrigin), b(rayDir));

    TraverseProp prop = TraverseProp(maxDepth, maxDistance, maxSearchDepth);

    return traverseRay(trav, prop);
}

// Intersection genShadowRay(Intersection curIntSec) {
//     NodeInfo light = lightData[0];

//     vec3 rayOrigin = vec(curIntSec.info.pos);
//     vec3 rayDir = normalize(vec(curIntSec.info.pos) - vec(light.pos));

//     // rayOrigin += rayDir * curIntSec.info.span;

//     Ray ray = Ray(rayOrigin, rayDir);
//     TraverseProp prop = TraverseProp(maxDepth, maxDistance, maxSearchDepth);

//     Intersection intSec = traverseRay(ray, prop);
//     if (octreeData[intSec.info.index].nodeType != 3) {
//         intSec.intersect = false;
//     }

//     if (gl_FragCoord.x < 1 && gl_FragCoord.y < 1) {
//         // debugPrintfEXT("\n%d", intSec.info.index);
//     }

//     return intSec;
// }

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
    TreeNode node = octreeData[intSec.traverse.index];
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
