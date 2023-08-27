#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable
// #extension EXT_gpu_shader4 : require

#define MAX_DEPTH 6
#define MAX_STEP 50
#define TEXTURE_ALIGN 16

layout (location = 0) in vec4 screen_pos;
layout (location = 1) flat in vec4 pos_on_edge;
layout (location = 2) in vec4 world_pos; // pos_on_edge + local_pos
layout (location = 3) in vec2 out_uv;
layout (location = 4) flat in uint loc_idx;

layout (location = 0) out vec4 frag_color;

#define mask_to_vec(mask) (vec3(mask & 1, (mask & 2) >> 1, (mask & 4) >> 2))
#define vec_to_mask(vec) ((uint(vec.x) << 0) | (uint(vec.y) << 1) | (uint(vec.z) << 2))

#define is_leaf(node) ((node & 16777216) > 0)
#define is_subdiv(node) ((node & 33554432) > 0)
#define child_idx(node, mask) ((node & 65535) + mask)

#define pos_to_px(pos) (vec2(pos.x, pos.y + (pos.z * TEXTURE_ALIGN)))

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
    uint last_hit_idx[16];

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
layout (set = 3, binding = 0) uniform sampler2D brick_texture;

vec3 rayCubeIntersect(vec3 origin, vec3 dir, vec3 inv_ray_dir, float span) {
    float size_cp = span * 0.5;
    vec3 inv_pos = sign(dir) * (origin - size_cp) - size_cp;

    return -inv_pos * inv_ray_dir;
}

// todo: fix shader, dunno maybe look into cpu side
// todo: edit to support sdf with jfa
// todo: be happy :)
void main() {
    frag_color = vec4(0);

    vec3 world_pos = world_pos.xyz;

    float span = loc_info[loc_idx].span;

    vec3 ray_dir = normalize(world_pos - uniform_buffer.cam_front.xyz);
    vec3 inv_ray_dir = 1.0 / max(abs(ray_dir), 0.001);
    Ray ray = Ray(world_pos, ray_dir, inv_ray_dir);

    vec3 base_pos_on_edge = pos_on_edge.xyz;

    vec3 hit = rayCubeIntersect(world_pos - base_pos_on_edge, ray.dir, ray.inv_ray_dir, span);
    vec3 hit_mask_vec = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));
    float len = dot(hit, hit_mask_vec);

    vec3 pos_on_edge = floor(world_pos);
    vec3 local_pos = world_pos - pos_on_edge;

    if (gl_FragCoord.xy.x < 4 && gl_FragCoord.xy.y < 4) {
        //debugPrintfEXT("\n%f %d %d", span, depth, loc_info[loc_idx].parent_list[0]);
    }

    len = 50.0;

    float max_len = len;
    float dist = 0.0;

    vec2 base_pos = vec2(0); // todo
    // vec2 max_pos = vec2(TEXTURE_ALIGN, TEXTURE_ALIGN + (TEXTURE_ALIGN * TEXTURE_ALIGN)) + base_pos;

    vec4 col = texelFetch(brick_texture, ivec2(pos_to_px(pos_on_edge)), 0);

    bool out_parent = false;

    for (uint iter = 0; iter < MAX_STEP; iter += 1) {
        if (col.w == 0.0) {
            frag_color = vec4(0, 1, 0, 0);
            return;
        }

        hit = rayCubeIntersect(local_pos, ray.dir, ray.inv_ray_dir, 1.0);
        hit_mask_vec = vec3(lessThan(hit, min(hit.yzx, hit.zxy)));

        len = dot(hit, hit_mask_vec);

        vec3 local_pos_on_edge = hit_mask_vec * sign(ray.dir);

        local_pos += ray.dir * len - local_pos_on_edge;
        pos_on_edge += local_pos_on_edge;

        dist += len;

        local_pos += len * ray.dir;

        col = texelFetch(brick_texture, ivec2(pos_to_px(pos_on_edge)), 0);
        out_parent = dist > max_len;
    }
}