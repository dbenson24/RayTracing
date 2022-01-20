use std::ops::{Index, IndexMut};

use glam::Vec3;

pub struct OrthoNormalBasis {
    axis: [Vec3; 3],
}

impl OrthoNormalBasis {
    pub fn from_w(n: &Vec3) -> Self {
        let mut axis = [Vec3::ZERO; 3];
        let w = n.normalize();
        axis[2] = w;
        let a = if w.x.abs() > 0.9 { Vec3::Y } else { Vec3::X };
        let v = w.cross(a).normalize();
        axis[1] = v;
        axis[0] = w.cross(v);

        Self { axis }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }
    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }
    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local(&self, a: &Vec3) -> Vec3 {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }
}

impl Index<usize> for OrthoNormalBasis {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.axis[index]
    }
}

impl IndexMut<usize> for OrthoNormalBasis {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.axis[index]
    }
}
