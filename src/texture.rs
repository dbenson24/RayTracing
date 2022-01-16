use std::sync::Arc;

use crate::color::Color;
use glam::Vec3;

pub trait Texture: Sync + Send {
    fn value(&self, u: f32, v: f32, p: &Vec3) -> Color;
}

pub struct SolidTex {
    pub color: Color,
}

impl SolidTex {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Texture for SolidTex {
    fn value(&self, u: f32, v: f32, p: &Vec3) -> Color {
        self.color
    }
}

pub struct CheckerTex {
    pub even: Arc<dyn Texture>,
    pub odd: Arc<dyn Texture>,
}

impl CheckerTex {
    pub fn new(even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self { even, odd }
    }
    pub fn from_colors(even: Color, odd: Color) -> Self {
        Self::new(Arc::new(SolidTex::new(even)), Arc::new(SolidTex::new(odd)))
    }
}

impl Texture for CheckerTex {
    fn value(&self, u: f32, v: f32, p: &Vec3) -> Color {
        let mut sines = 1.;
        for i in 0..3 {
            sines *= (p[i] * 10.).sin()
        }

        if sines < 0. {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
