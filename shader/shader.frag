#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
#extension GL_EXT_debug_printf : enable

#define MAX_DEPTH 6

layout (location = 0) in vec4 screen_pos;
layout (location = 1) in vec4 world_pos;
layout (location = 2) in vec2 out_uv;
layout (location = 3) flat in uint loc_idx;

layout (location = 0) out vec4 frag_color;

#define bitset(num, bit) (num | 1 << bit)
#define bitclear(num, bit) (num & !(1 << bit))
#define bitflip(num, bit) (num ^ 1 << bit)
#define bitread(num, bit) ((num >> bit) & 1)
#define bitcheck(num, bit) (bitread(num, bit) != 0)
#define create_mask(s, e) ((!0 >> (32 - (e - s))) << s)
#define read_bitrange(num, s, e) ((num & create_mask(s, e)) >> s)
#define mask_to_vec(mask) (vec3(bitread(mask, 0), bitread(mask, 2), bitread(mask, 1)))
#define vec_to_mask(vec) () // TO DO

struct PosInfo {
    vec4 local_pos;
    vec4 pos_on_edge;
    uint depth;
};

struct Ray {
    vec3 origin;
    vec3 dir;

    vec3 invRayDir; // Used for RayCube Intersection
};

struct BranchInfo {
    uint node;
    uint parent;

    uint index;
    uint parent_index;

    float span;

    uint mask_info;

    // hit_list
    // last_hit_idx

    uint padding [2];
};

struct LocInfo {
    vec4 pos_on_edge;

    uint node;
    uint parent;

    uint index;
    uint parent_index;

    float span;
    uint depth;

    uint padding [2];
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

void main() {
    frag_color = vec4(loc_info[0].depth / 50);

    vec3 ray_dir = normalize(world_pos.xyz - uniform_buffer.pos.xyz);
    vec3 local_pos = world_pos % loc_info[loc_idx].span;

    PosInfo pos_info = PosInfo(
        local_pos,
        loc_info[loc_idx].pos_on_edge,
        loc_info[loc_idx].depth
    );

    BranchInfo 

    uint hit_list = [0, 0, 0, 0];



    // gl_FragColor = vec4(0,1,0,0);

    // debugPrintfEXT("\n%d", loc_info[0].depth);
}
