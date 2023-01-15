#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Uniform {
    pub time: u32,

    pub raw_field_of_view: f32,
    pub max_ray_length: u32,

    pub rot_horizontal: f32,
    pub rot_vertical: f32,

    pub octree_root_index: u32,
    
    pub node_at_pos: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Simple Data storage
impl Uniform {
    pub fn empty() -> Uniform {
        Uniform {
            time: 0,
            
            raw_field_of_view: 60.0,
            max_ray_length: 300,

            rot_horizontal: 0.0,
            rot_vertical: 0.0,

            octree_root_index: 0,

            node_at_pos: 0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}