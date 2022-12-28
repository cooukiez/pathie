use cgmath::Vector3;

pub struct Service { }

impl Service {
    pub fn pos_to_index(pos: &Vector3<f32>, side_len: i32, ) -> u32 {
        (((pos.x as i32) % side_len) + ((pos.x as i32) % side_len) * side_len * side_len + ((pos.x as i32) % side_len) * side_len) as u32 
    }

    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] { 
        vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len()))
    }

    pub fn check_number_in_range(range: [f32; 2], number: f32, ) -> bool {
        if range[0] < range[1] { range[0] <= number && number <= range[1] } else { range[1] <= number && number <= range[0] }
    }

    pub fn check_in_volume(first: &Vector3<f32>, sec: &Vector3<f32>, check: &Vector3<f32>, ) -> bool {
        Self::check_number_in_range([first.x, sec.x], check.x, ) && Self::check_number_in_range([first.y, sec.y], check.y, ) && Self::check_number_in_range([first.z, sec.z], check.z, )
    }

    pub fn convert_index_to_pos(index: u32, side_len: u32, ) -> Vector3<f32> {
        let x = ((index % (side_len * side_len)) % side_len) as f32;
        let y = (index / (side_len * side_len)) as f32;
        let z = ((index % (side_len * side_len)) / side_len) as f32;
        Vector3::new(x, y, z)
    }
}