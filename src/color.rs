use image::Rgb;

pub type Color = glam::Vec3;

pub trait RGB {
    fn r(&self) -> f32;
    fn g(&self) -> f32;
    fn b(&self) -> f32;
    fn set_r(&mut self, r: f32);
    fn set_g(&mut self, g: f32);
    fn set_b(&mut self, b: f32);

    fn to_px(&self, samples: usize) -> Rgb<u8> {
        let scale = 1. / samples as f32;
        let r: u8 = ((self.r() * scale).sqrt() * 255.9999) as u8;
        let g: u8 = ((self.g() * scale).sqrt() * 255.9999) as u8;
        let b: u8 = ((self.b() * scale).sqrt() * 255.9999) as u8;
        Rgb([r, g, b])
    }
}

impl RGB for Color {
    fn r(&self) -> f32 {
        self.x
    }

    fn g(&self) -> f32 {
        self.y
    }

    fn b(&self) -> f32 {
        self.z
    }

    fn set_r(&mut self, r: f32) {
        self.x = r
    }

    fn set_g(&mut self, g: f32) {
        self.y = g
    }

    fn set_b(&mut self, b: f32) {
        self.z = b
    }
}
