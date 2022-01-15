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
use std::{time::Instant, rc::Rc, borrow::Borrow};

use crate::{camera::Camera, color::RGB};
use crate::{
    material::{Dielectric, Lambertian, Material, Metal, ToWithMat, WithMat},
    world::World,
};

const PI: f32 = 3.1415926535;

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
    println!("Setup");
    let aspect_ratio: f32 = 16.0 / 9.0;
    let width = 1920;
    let height = (width as f32 / aspect_ratio) as usize;
    let mut pixels = vec![Color::default(); width * height];

    let samples_per_px = 100;
    let max_bounces = 50;
    let R = (PI / 4.).cos();
    let aspect_ratio = 16.0 / 9.0;
    let aperture = 0.1;

    let mut world = World::new(vec![]);

    let mut pairs = vec![];

    for a in -11..11 {
        for b in -11..11 {
            let center = Vec3::new(a as f32 + 0.9 * random(), 0.2, b as f32 + 0.9 * random());
            if (center - Vec3::new(4., 0.2, 0.)).length() > 0.9 {
                let mat = rand_mat();
                let sphere = Sphere::new(center, 0.2);
                pairs.push((sphere, mat));
            }
        }
    }

    for (sphere, mat) in &pairs {
        world.objs.push(sphere.with_mat(mat.borrow()))
    }

    let mat_ground = Lambertian::new(Vec3::new(0.5, 0.5, 0.5));

    let ground = Sphere::new(Vec3::new(0., -1000.5, -1.), 1000.);

    world.objs.push(ground.with_mat(&mat_ground));

    let glass = Dielectric::new(1.5);
    let glass_sphere = Sphere::new(Vec3::new(0., 1., 0.), 1.0);
    world.objs.push(glass_sphere.with_mat(&glass));

    let brown = Lambertian::new(Vec3::new(0.4, 0.2, 0.1));
    let brown_sphere = Sphere::new(Vec3::new(-4., 1., 0.), 1.0);
    world.objs.push(brown_sphere.with_mat(&brown));

    let metal = Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0);
    let metal_sphere = Sphere::new(Vec3::new(4., 1., 0.), 1.);
    world.objs.push(metal_sphere.with_mat(&metal));



    // Camera
    let origin = Vec3::new(13., 2., 3.);
    let lookat = Vec3::new(0., 0., 0.);
    // let dist_to_focus = (lookat - origin).length();
    let dist_to_focus = 10.;
    let up = Vec3::new(0., 1., 0.);
    let camera = Camera::new(origin, lookat, up, 40., aspect_ratio, aperture, dist_to_focus);

    world.build();

    println!("Begin Tracing");

    let now = Instant::now();
    pixels.par_iter_mut().enumerate().for_each(|(i, px)| {
        let x = i % width;
        let y = (height - 1) - (i / width);
        for _ in 0..samples_per_px {
            let u = (x as f32 + random()) / (width - 1) as f32;
            let v = (y as f32 + random()) / (height - 1) as f32;
            let ray = camera.get_ray(u, v);
            *px += ray_color(&ray, &world, max_bounces);
        }
    });

    let elapsed = now.elapsed();
    println!("Done Tracing in {} ms", elapsed.as_millis());

    let image = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        let i = (x + (y * width as u32)) as usize;
        let c = pixels[i];
        c.to_px(samples_per_px)
    });

    image.save("out.png").expect("Image to save");
    println!("Image written to out.png");
}

fn random() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0)
}

fn rand_range(min: f32, max: f32) -> f32 {
    min + ((max - min) * random())
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

fn rand_in_disk() -> Vec3 {
    loop {
        let x = Vec3::new(rand_range(-1., 1.), rand_range(-1., 1.), 0.);
        if x.length_squared() < 1. {
            return x;
        }
    }
}

fn rand_unit_vector() -> Vec3 {
    rand_in_sphere().normalize()
}

fn rand_vec3() -> Vec3 {
    Vec3::new(random(), random(), random())
}

fn rand_vec3_range(min: f32, max: f32) -> Vec3 {
    let diff = max - min;
    Vec3::splat(min) + (diff * rand_vec3())
}

fn reflect(d: Vec3, n: Vec3) -> Vec3 {
    d - (2. * (d.dot(n)) * n)
}

fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}

fn reflectance(cosine: f32, refract_idx: f32) -> f32 {
    // Use Schlick's approximation for reflectance.
    let mut r0 = (1. - refract_idx) / (1. + refract_idx);
    r0 = r0 * r0;
    r0 + (1. - r0) * (1. - cosine).powf(5.)
}

fn rand_mat() -> Rc<dyn Material> {
    let choose_mat = random();
    if choose_mat < 0.8 {
        // diffuse
        let albedo = rand_vec3() * rand_vec3();
        Rc::new(Lambertian::new(albedo))
    } else if choose_mat < 0.95 {
        let albedo = rand_vec3_range(0.5, 1.);
        let fuzz = rand_range(0., 0.5);
        Rc::new(Metal::new(albedo, fuzz))
    } else {
        Rc::new(Dielectric::new(1.5))
    }
}