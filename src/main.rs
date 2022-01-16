mod camera;
mod color;
mod material;
mod mesh;
mod texture;
mod world;
mod instance;

use bvh::{
    bvh::BVH,
    ray::{Intersection, IntersectionRay, Ray},
    sphere::Sphere,
};
use color::Color;
use glam::{Vec3, Quat};
use image::{ImageBuffer, Rgb};
use rand::Rng;
use rayon::prelude::*;
use std::{
    borrow::Borrow, f32::consts::PI, fs::File, io::BufReader, rc::Rc, sync::Arc, time::Instant,
};
use texture::CheckerTex;

use crate::{camera::Camera, color::RGB, mesh::Mesh, instance::Instance};
use crate::{
    material::{Dielectric, Lambertian, Material, Metal, ToWithMat, WithMat},
    world::World,
};
use obj::{load_obj, Obj};



fn main() {
    render_trimesh()
}

fn random_sphere_world() -> World {
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
        world.objs.push(sphere.with_mat(mat.clone()))
    }

    let mat_ground = Arc::new(Lambertian::from_tex(Arc::new(CheckerTex::from_colors(
        Vec3::new(0.2, 0.3, 0.1),
        Vec3::new(0.9, 0.9, 0.9),
    ))));
    let ground = Sphere::new(Vec3::new(0., -1000.5, -1.), 1000.);

    world.objs.push(ground.with_mat(mat_ground));

    let glass = Arc::new(Dielectric::new(1.5));
    let glass_sphere = Sphere::new(Vec3::new(0., 1., 0.), 1.0);
    world.objs.push(glass_sphere.with_mat(glass));

    let brown = Arc::new(Lambertian::new(Vec3::new(0.4, 0.2, 0.1)));
    let brown_sphere = Sphere::new(Vec3::new(-4., 1., 0.), 1.0);
    world.objs.push(brown_sphere.with_mat(brown));

    let metal = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
    let metal_sphere = Arc::new(Sphere::new(Vec3::ZERO, 1.));
    let metal_sphere = Instance::from_trs(metal_sphere, Vec3::new(4., 1., 0.), Quat::IDENTITY, Vec3::new(1.0, 3.0, 1.0));
    world.objs.push(metal_sphere.with_mat(metal));

    world.build();

    world
}

fn render_trimesh() {
    println!("Setup");
    let height = 720;
    let origin = Vec3::new(3., 6., 13.);
    let lookat = Vec3::new(0., 0., 0.);
    let vfov = 70.;
    let mut world = World::new(vec![]);
    let mesh = Arc::new(Mesh::from_file("teapot.obj"));
    let mat = Arc::new(Lambertian::new(Vec3::new(0.7, 0.6, 0.5)));
    let metal = Arc::new(Metal::new(Vec3::new(0.9, 0.1, 0.1), 0.));
    let mesh_1 = Instance::from_trs(mesh.clone(), Vec3::new(-1.5, 0., 0.), Quat::IDENTITY, Vec3::new(2.0, 2.0, 2.0));
    let mesh_2 = Instance::from_t(mesh.clone(), Vec3::new(5.5, 0., 0.));
    
    world.objs.push(mesh_1.with_mat(mat));
    world.objs.push(mesh_2.with_mat(metal));
    world.build();
    for obj in &world.objs {
        dbg!(obj.node_index);
    }

   
    world.render("two_spheres.png", height, origin, lookat, vfov);
}

fn render_random_spheres() {
    println!("Setup");
    let height = 480;
    let world = random_sphere_world();
    let vfov = 20.;
    
    let origin = Vec3::new(13., 2., 3.);
    let lookat = Vec3::new(0., 0., 0.);

    world.render("random_spheres.png", height, origin, lookat, vfov);
    
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

fn rand_mat() -> Arc<dyn Material> {
    let choose_mat = random();
    if choose_mat < 0.8 {
        // diffuse
        let albedo = rand_vec3() * rand_vec3();
        Arc::new(Lambertian::new(albedo))
    } else if choose_mat < 0.95 {
        let albedo = rand_vec3_range(0.5, 1.);
        let fuzz = rand_range(0., 0.5);
        Arc::new(Metal::new(albedo, fuzz))
    } else {
        Arc::new(Dielectric::new(1.5))
    }
}
