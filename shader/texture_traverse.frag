#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable
// #extension EXT_gpu_shader4 : require

#define MAX_DEPTH 6
#define MAX_STEP 10

layout (location = 0) in vec4 screen_pos;
layout (location = 1) in vec2 out_uv;
layout (location = 2) in vec4 world_pos; // pos_on_edge + local_pos
layout (location = 3) flat in uint loc_idx;

layout (location = 0) out vec4 frag_color;

#define bitset(num, bit) (num | 1 << bit)
#define bitclear(num, bit) (num & !(1 << bit))
#define bitflip(num, bit) (num ^ 1 << bit)

#define bitread(num, bit) ((num >> bit) & 1)
#define bitcheck(num, bit) (bitread(num, bit) != 0)

#define create_mask(s, e) ((~0 >> (32 - (e - s))) << s)
#define read_bitrange(num, s, e) ((num & create_mask(s, e)) >> s)

#define mask_to_vec(mask) (vec4(bitread(mask, 0), bitread(mask, 1), bitread(mask, 2), 0))
#define vec_to_mask(vec) ((uint(vec.x) << 0) | (uint(vec.y) << 1) | (uint(vec.z) << 2))

#define is_leaf(node) (bitcheck(node, 24))
#define is_subdiv(node) (bitcheck(node, 25))

#define pos_to_px(pos) (vec2((pos.y * span) + pos.x, pos.z))

struct PosInfo {
    vec4 local_pos;
    vec4 pos_on_edge;
    uint depth;
};

struct Ray {
    vec4 origin;
    vec4 dir;

    vec4 inv_ray_dir; // Used for RayCube Intersection
};

struct LocInfo {
    // For proper alignment set depth to 16
    uint parent_list[16];
    uint depth;
    float span;

    uint padding[2];
};

layout (set = 0, binding = 0) uniform Uniform {
    mat4 view_proj;
    vec4 pos;

    vec4 cam_pos;
    vec4 cam_front;
    vec4 cam_up;
    vec4 look_dir;

    vec2 res;
    vec2 mouse_delta;
    vec2 mouse_pos;

    float root_span;
    uint time;

    uint padding[2];
} uniform_buffer;

layout (set = 1, binding = 0) buffer NodeData { uint node_data[4096]; };
layout (set = 2, binding = 0) buffer LocationData { LocInfo loc_info[16]; };
layout (set = 3 /* ??? */, binding = 0) uniform sampler2D brick_texture;

vec4 rayCubeIntersect(vec4 origin, vec4 dir, vec4 inv_ray_dir, float span) {
    return -(sign(dir) * (origin - span * 0.5) - span * 0.5) * inv_ray_dir;
}

void main() {
    vec4 ray_dir = vec4(normalize(world_pos.xyz - uniform_buffer.pos.xyz), 0);
    // vec4 local_pos = world_pos % loc_info[loc_idx].span;

    float span = loc_info[loc_idx].span;
    int depth = int(loc_info[loc_idx].depth);

    span = 8.0;
    depth = 0;

    vec4 local_pos = mod(world_pos, span);
    frag_color = local_pos;
    vec4 pos_on_edge = world_pos - local_pos;

    vec4 inv_ray_dir = vec4(1.0 / max(abs(ray_dir.xyz), 0.001), 0);
    Ray ray = Ray(world_pos, ray_dir, inv_ray_dir);

    vec2 base_pos = vec2(0); // todo

    vec2 px = base_pos + pos_to_px(local_pos);
    vec4 col = texelFetch(brick_texture, ivec2(px), 0);

    uint pos_mask;
    uint dir_mask;

    vec4 test;

    for (uint iter = 0; iter < MAX_STEP; iter += 1) {
        if (col.w > 0.0) {
            return;
        }

        vec4 hit = rayCubeIntersect(local_pos, ray.dir, ray.inv_ray_dir, span * 0.5);
        vec4 hit_mask_vec = vec4(lessThan(hit.xyz, min(hit.yzx, hit.zxy)), 0);

        float len = dot(hit, hit_mask_vec);

        px += pos_to_px(hit_mask_vec);
        local_pos += len * ray.dir;

        vec4 col = texelFetch(brick_texture, ivec2(px), 0);
    }

    frag_color = col;

    //frag_color *= test;

    // necessary
    // hitlist = 4 * 4by
    // last_hit_idx = 4by
    // child_offset = 4by

    // 0. get all children
    // 1. determine all possible hit cand.
    // 2. calc next hit with last_hit_idx and hit_list
    // 3. get node
    // 4. add new branch
    // 

    // gl_FragColor = vec4(0,1,0,0);

    // debugPrintfEXT("\n%d", loc_info[0].depth);
}