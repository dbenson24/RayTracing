mod camera;
mod color;
mod material;
mod world;

use bvh::{
    bvh::BVH,
    ray::{Intersection, IntersectionRay, Ray},
    sphere::Sphere,
};
use color::Color;
use glam::Vec3;
use image::{ImageBuffer, Rgb};
use rand::Rng;
use rayon::prelude::*;

use crate::{world::World, material::{Lambertian, Metal, WithMat}};
use crate::{camera::Camera, color::RGB};

fn ray_color(ray: &Ray, world: &World, depth: usize) -> Color {
    if depth == 0 {
        return Vec3::ZERO;
    }

    if let Some((obj, intersection)) = world.first_intersection(ray, 0.001, f32::INFINITY) {
        if let Some((child_ray, attenuation)) = obj.scatter(ray, &intersection) {
            ray_color(&child_ray, world, depth - 1) * attenuation
        } else {
            Vec3::ZERO
        }
    } else {
        let t = 0.5 * (ray.direction.y + 1.0);
        (1. - t) * Vec3::ONE + t * Vec3::new(0.5, 0.7, 1.0)
    }
}


fn main() {
    println!("Hello, world!");
    let aspect_ratio: f32 = 16.0 / 9.0;
    let width = 1920;
    let height = (width as f32 / aspect_ratio) as usize;
    let mut pixels = vec![Color::default(); width * height];

    let samples_per_px = 100;
    let max_bounces = 50;

    let mut world = World::new(vec![]);
    let mat_ground = Lambertian::new(Vec3::new(0.8, 0.8, 0.0));
    let mat_center = Lambertian::new(Vec3::new(0.7, 0.3, 0.3));
    let mat_left = Metal::new(Vec3::new(0.8, 0.8, 0.8));
    let mat_right = Metal::new(Vec3::new(0.8, 0.6, 0.2));

    let ground = Sphere::new(Vec3::new(0., -100.5, -1.), 100.);
    let center = Sphere::new(Vec3::new(0., 0., -1.), 0.5);
    let left = Sphere::new(Vec3::new(-1., 0., -1.), 0.5);
    let right = Sphere::new(Vec3::new(1., 0., -1.), 0.5);

    let g = WithMat::new(&ground, &mat_ground);
    let c = WithMat::new(&center, &mat_center);
    let l = WithMat::new(&left, &mat_left);
    let r = WithMat::new(&right, &mat_right);

    world.objs.push(&g);
    world.objs.push(&c);
    world.objs.push(&l);
    world.objs.push(&r);

    // Camera
    let origin = Vec3::new(0., 0., 0.);
    let camera = Camera::new(origin);

    pixels.par_iter_mut().enumerate().for_each(|(i, px)| {
        let x = i % width;
        let y = (height - 1) - (i / width);
        for _ in 0..samples_per_px {
            let u = (x as f32 + rand()) / (width - 1) as f32;
            let v = (y as f32 + rand()) / (height - 1) as f32;
            let ray = camera.get_ray(u, v);
            *px += ray_color(&ray, &world, max_bounces);
        }
    });

    let image = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        let i = (x + (y * width as u32)) as usize;
        let c = pixels[i];
        c.to_px(samples_per_px)
    });

    image.save("out.png").expect("Image to save");
}


fn rand() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}

fn rand_range(min: f32, max: f32) -> f32 {
    min + ((max - min) * rand())
}

fn rand_in_sphere() -> Vec3 {
    loop {
        let x = Vec3::new(
            rand_range(-1., 1.),
            rand_range(-1., 1.),
            rand_range(-1., 1.),
        );
        if x.length_squared() < 1. {
            return x;
        }
    }
}

fn rand_unit_vector() -> Vec3 {
    rand_in_sphere().normalize()
}

fn reflect(d: Vec3, n: Vec3) -> Vec3 {
    d - (2. * (d.dot(n)) * n)
}
