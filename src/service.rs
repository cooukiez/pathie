pub struct Service { }

impl Service {
    pub fn vec_to_array<Type, const Length: usize>(vec: Vec<Type>) -> [Type; Length] { 
        vec.try_into().unwrap_or_else(| vec: Vec<Type> | panic!("ERR_INVALI_LEN -> Expected {} | Got {}", Length, vec.len())) 
    }

    pub fn check_boundary(boundary: [f32; 2], number: f32) -> bool {
        if boundary[0] < boundary[1] { boundary[0] < number && number < boundary[1] }
        else { boundary[1] < number && number < boundary[0] }
    }
}