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

#define mask_to_vec(mask) (vec4(bitread(mask, 0), bitread(mask, 2), bitread(mask, 1), 0))
#define vec_to_mask(vec) ((uint(vec.x) << 0) | (uint(vec.z) << 1) | (uint(vec.y) << 2))

#define is_leaf(node) (bitcheck(node, 24))
#define is_subdiv(node) (bitcheck(node, 25))

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

uint get_first_child_idx(uint parent) {
    return read_bitrange(parent, 0, 15);
}

uint get_child(uint mask, uint parent) {
    return node_data[get_first_child_idx(parent) + mask];
}

vec4 rayCubeIntersect(vec4 origin, vec4 dir, vec4 inv_ray_dir, float span) {
    return -(sign(dir) * (origin - span * 0.5) - span * 0.5) * inv_ray_dir;
}

void main() {
    vec4 ray_dir = vec4(normalize(world_pos.xyz - uniform_buffer.pos.xyz), 0);
    // vec4 local_pos = world_pos % loc_info[loc_idx].span;

    float span = loc_info[loc_idx].span;
    uint depth = loc_info[loc_idx].depth;

    span = 8.0;
    depth = 3;

    vec4 local_pos = mod(world_pos, span);
    frag_color = local_pos;
    vec4 pos_on_edge = world_pos - local_pos;

    PosInfo pos_info = PosInfo(
        local_pos,
        pos_on_edge,
        depth
    );

    // depth set to 16 for proper alignment
    uint parent_list[16] = loc_info[loc_idx].parent_list;
    // set to something that is not an actual index to indicate
    // wether there is an active index in use or not
    uint last_hit_idx[16] = uint[](8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8);
    vec4 inv_ray_dir = vec4(1.0 / max(abs(ray_dir.xyz), 0.001), 0);
    Ray ray = Ray(world_pos, ray_dir, inv_ray_dir);

    uint pos_mask;
    uint dir_mask;

    vec4 test;

    for (uint iter = 0; iter < MAX_STEP; iter += 1) {
        // get if not already visited by checking if index is valid
        if (last_hit_idx[depth] > 7) {
            // Get in which child we are
            last_hit_idx[depth] = vec_to_mask(step(local_pos, vec4(span * 0.5)));
        }

        pos_mask = last_hit_idx[depth];

        // Update local pos and pos on edge
        local_pos -= mask_to_vec(pos_mask) * span;
        pos_on_edge += mask_to_vec(pos_mask) * span;

        // Get next hit
        vec4 hit = rayCubeIntersect(local_pos, ray.dir, ray.inv_ray_dir, span * 0.5);
        dir_mask = vec_to_mask(vec4(lessThan(hit.xyz, min(hit.yzx, hit.zxy)), 0));

        uint new_mask = pos_mask | dir_mask;
        bool out_parent = new_mask == pos_mask;

        if (out_parent) {
            last_hit_idx[depth] = 8;

            local_pos += mask_to_vec(pos_mask) * span;
            pos_on_edge -= mask_to_vec(pos_mask) * span;

            depth -= 1;
            span *= 2;
        } else {
            uint node = get_child(parent_list[depth - 1], new_mask);
            last_hit_idx[depth] = new_mask;

            if (is_subdiv(node)) {
                parent_list[depth] = node;

                depth += 1;
                span *= 0.5;
            }
            if (is_leaf(node)) {
                test = vec4(0.1, 1, 0.1, 0);
                return;
            }
        }
    }

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