#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable
// #extension EXT_gpu_shader4 : require

#define MAX_DEPTH 6
#define MAX_STEP 40

layout (location = 0) in vec4 screen_pos;
layout (location = 1) in vec2 out_uv;
layout (location = 2) in vec4 world_pos; // pos_on_edge + local_pos
layout (location = 3) flat in uint loc_idx;

layout (location = 0) out vec4 frag_color;

#define mask_to_vec(mask) (vec3(mask & 1, (mask & 2) >> 1, (mask & 4) >> 2))
#define vec_to_mask(vec) ((uint(vec.x) << 0) | (uint(vec.y) << 1) | (uint(vec.z) << 2))

#define is_leaf(node) ((node & 16777216) > 0)
#define is_subdiv(node) ((node & 33554432) > 0)
#define child_idx(node, mask) ((node & 65535) + mask)

struct PosInfo {
    vec3 local_pos;
    vec3 pos_on_edge;
    uint depth;
};

struct Ray {
    vec3 origin;
    vec3 dir;

    vec3 inv_ray_dir; // Used for RayCube Intersection
};

struct LocInfo {
    // For proper alignment set depth to 16
    uint parent_list[16];
    uint padding[2];
    uint depth;
    float span;
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

uint get_child(uint parent, uint mask) {
    return node_data[child_idx(parent, mask)];
}

vec3 rayCubeIntersect(vec3 origin, vec3 dir, vec3 inv_ray_dir, float span) {
    float size_cp = span * 0.5;
    vec3 inv_pos = sign(dir) * (origin - size_cp) - size_cp;

    return -inv_pos * inv_ray_dir;
}

void main() {
    vec3 world_pos = world_pos.xyz;
    vec3 ray_dir = normalize(world_pos - uniform_buffer.pos.xyz);
    // vec4 local_pos = world_pos % loc_info[loc_idx].span;

    float span = loc_info[loc_idx].span;
    int depth = int(loc_info[loc_idx].depth);

    //span = 8.0;
    //depth = 0;

    vec3 local_pos = mod(world_pos, span);
    frag_color = vec4(local_pos, 0);
    vec3 pos_on_edge = world_pos - local_pos;

    // debugPrintfEXT("\n%d", parent_list[0]);

    // debugPrintfEXT("\nc%d", get_child(parent_list[0], 0));

    // depth set to 16 for proper alignment
    uint parent_list[16] = loc_info[loc_idx].parent_list;
    // set to something that is not an actual index to indicate
    // wether there is an active index in use or not
    uint last_hit_idx[16] = uint[16](8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8);
    vec3 inv_ray_dir = 1.0 / max(abs(ray_dir), 0.001);
    Ray ray = Ray(world_pos, ray_dir, inv_ray_dir);

    uint a = node_data[child_idx(0, 0)];
    uint b = uint(is_subdiv(node_data[child_idx(node_data[0], 0)]));

    uint pos_mask;
    uint dir_mask;
    uint _i_debug = 0;

    for (uint iter = 0; iter < MAX_STEP; iter += 1) {
        frag_color = vec4(world_pos,0);
        span *= 0.5;

        // get if not already visited by checking if index is valid
        if (last_hit_idx[depth] > 7) {
            // Get in which child we are
            vec3 stepped = step(vec3(span), local_pos);

            pos_mask = vec_to_mask(stepped);
            last_hit_idx[depth] = pos_mask;
        }

        _i_debug = parent_list[depth - 1];

        vec3 vec_pos_mask = mask_to_vec(pos_mask);
        vec3 local_pos_on_edge = vec_pos_mask * span;

        // Update local pos and pos on edge
        local_pos -= local_pos_on_edge;
        pos_on_edge += local_pos_on_edge;

        // Get next hit
        vec3 hit = rayCubeIntersect(local_pos, ray.dir, ray.inv_ray_dir, span);
        vec3 hit_mask_vec = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));

        float len = dot(hit, hit_mask_vec);

        vec3 new_voxel = hit_mask_vec * sign(ray.dir) * span;

        local_pos += ray.dir * len - new_voxel;
        pos_on_edge += new_voxel;

        // get dir mask without sign information
        dir_mask = vec_to_mask(hit_mask_vec);
        // extract sign information
        float dir_sign = dot(hit_mask_vec, sign(ray.dir));
        bool inv = dir_sign < 0;

        uint out_parent_raw = inv ? ~pos_mask & dir_mask : pos_mask & dir_mask;
        bool out_parent = out_parent_raw > 0;

        uint new_mask = pos_mask | dir_mask;

        // reset back to parent
        local_pos += local_pos_on_edge + new_voxel;
        pos_on_edge -= local_pos_on_edge - new_voxel;

        if (out_parent) {
            last_hit_idx[depth] = 8;

            depth -= 1;
            span *= 4; // we set the span at the start to the child span

            if (depth < 0) {
                frag_color = vec4(world_pos,0);
                return;
            }
        } else {
            uint node = get_child(parent_list[depth - 1], new_mask);
            last_hit_idx[depth] = new_mask;

            if (is_subdiv(node)) {
                parent_list[depth] = node;
                depth += 1;
                //debugPrintfEXT("\n%d", 1);
            }
            if (is_leaf(node)) {
                frag_color = vec4(0.1, 1, iter / MAX_STEP, 0);
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
}