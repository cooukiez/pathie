use cgmath::Vector3;

pub struct Service { }

impl Service {
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] { 
        vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len()))
    }

    pub fn check_boundary(boundary: [f32; 2], number: f32, ) -> bool {
        if boundary[0] < boundary[1] { boundary[0] <= number && number <= boundary[1] } else { boundary[1] <= number && number <= boundary[0] }
    }

    pub fn check_in_volume(first: &Vector3<f32>, sec: &Vector3<f32>, check: &Vector3<f32>, ) -> bool {
        Self::check_boundary([first.x, sec.x], check.x, ) && Self::check_boundary([first.y, sec.y], check.y, ) && Self::check_boundary([first.z, sec.z], check.z, )
    }
}
