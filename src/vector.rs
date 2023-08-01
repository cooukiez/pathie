/*

pub type TwoDeeVec<Type> = Vec<Vec<Type>>;
pub type ThreeDeeVec<Type> = Vec<Vec<Vec<Type>>>;

pub fn vec_two_dee<Type: std::clone::Clone>(side_len: usize, content: Type) -> TwoDeeVec<Type> {
    vec![vec![content; side_len]; side_len]
}

pub fn vec_three_dee<Type: std::clone::Clone>(side_len: usize, content: Type) -> ThreeDeeVec<Type> {
    vec![vec![vec![content; side_len]; side_len]; side_len]
}

*/

use nalgebra_glm::{Vec2, Vec3, Vec4};

pub trait Num {
    // Move Number back into boundary
    fn boundary(&self, min: Self, max: Self) -> Self;
}

impl Num for f32 {
    fn boundary(&self, min: Self, max: Self) -> Self {
        let mut corrected = self.clone();
        if self < &min {
            corrected = min
        }
        if self > &max {
            max
        } else {
            corrected
        }
    }
}

pub trait Vector {
    fn step(&self, edge: Self) -> Self;
    fn floor(&self) -> Self;
    fn sign(&self) -> Self;

    // Move Vector back into boundary
    fn boundary(&self, min: Self, max: Self) -> Self;

    fn any(&self, condition: fn(f32) -> bool) -> bool;

    // convert float to vec
    fn ftv(num: f32) -> Self;
}

impl Vector for Vec4 {
    fn step(&self, edge: Self) -> Self {
        Self::new(
            (edge.x < self.x).into(),
            (edge.y < self.y).into(),
            (edge.z < self.z).into(),
            (edge.w < self.w).into(),
        )
    }

    fn floor(&self) -> Self {
        Self::new(
            self.x.floor(),
            self.y.floor(),
            self.z.floor(),
            self.w.floor(),
        )
    }

    fn sign(&self) -> Self {
        Self::new(
            self.x.signum(),
            self.y.signum(),
            self.z.signum(),
            self.w.signum(),
        )
    }

    fn boundary(&self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.boundary(min.x, max.x),
            self.y.boundary(min.y, max.y),
            self.z.boundary(min.z, max.z),
            self.w.boundary(min.w, max.w),
        )
    }

    fn any(&self, condition: fn(f32) -> bool) -> bool {
        condition(self.x) || condition(self.y) || condition(self.z) || condition(self.w)
    }

    fn ftv(num: f32) -> Self {
        Self::new(num, num, num, num)
    }
}

impl Vector for Vec3 {
    fn step(&self, edge: Self) -> Self {
        Self::new(
            (edge.x < self.x).into(),
            (edge.y < self.y).into(),
            (edge.z < self.z).into(),
        )
    }

    fn floor(&self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    fn sign(&self) -> Self {
        Self::new(self.x.signum(), self.y.signum(), self.z.signum())
    }

    fn boundary(&self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.boundary(min.x, max.x),
            self.y.boundary(min.y, max.y),
            self.z.boundary(min.z, max.z),
        )
    }

    fn any(&self, condition: fn(f32) -> bool) -> bool {
        condition(self.x) || condition(self.y) || condition(self.z)
    }

    fn ftv(num: f32) -> Self {
        Self::new(num, num, num)
    }
}

impl Vector for Vec2 {
    fn step(&self, edge: Self) -> Self {
        Self::new((edge.x < self.x).into(), (edge.y < self.y).into())
    }

    fn floor(&self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    fn sign(&self) -> Self {
        Self::new(self.x.signum(), self.y.signum())
    }

    fn boundary(&self, min: Self, max: Self) -> Self {
        Self::new(self.x.boundary(min.x, max.x), self.y.boundary(min.y, max.y))
    }

    fn any(&self, condition: fn(f32) -> bool) -> bool {
        condition(self.x) || condition(self.y)
    }

    fn ftv(num: f32) -> Self {
        Self::new(num, num)
    }
}
