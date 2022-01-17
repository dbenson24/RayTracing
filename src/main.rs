mod camera;
mod color;
mod instance;
mod material;
mod mesh;
mod texture;
mod world;

use bvh::{
    bvh::BVH,
    ray::{Intersection, IntersectionRay, Ray},
    sphere::Sphere, aabb::Bounded,
};
use color::Color;
use glam::{Quat, Vec3};
use image::{ImageBuffer, Rgb};
use rand::Rng;
use rayon::prelude::*;
use std::{
    borrow::Borrow, f32::consts::PI, fs::File, io::BufReader, rc::Rc, sync::Arc, time::Instant,
};
use texture::CheckerTex;

use crate::{
    camera::Camera,
    color::RGB,
    instance::Instance,
    material::{DiffuseLight, Normals},
    mesh::Mesh,
};
use crate::{
    material::{Dielectric, Lambertian, Material, Metal, ToWithMat, WithMat},
    world::World,
};
use obj::{load_obj, Obj};

fn main() {
    cornell_box()
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
    let metal_sphere = Instance::from_trs(
        metal_sphere,
        Vec3::new(4., 1., 0.),
        Quat::IDENTITY,
        Vec3::new(1.0, 3.0, 1.0),
    );
    world.objs.push(metal_sphere.with_mat(metal));

    world.build();

    world
}

fn render_trimesh() {
    println!("Setup");
    let height = 720;
    let origin = Vec3::new(3., 6., 13.);
    let lookat = Vec3::new(0., 0., 0.);
    let vfov = 50.;
    let mut world = World::new(vec![]);
    let mesh = Arc::new(Mesh::from_file("teapot.obj", true));
    let sphere = Mesh::from_file("sphere.obj", true);
    // let sphere = Sphere::new(Vec3::ZERO, 1.0);
    // let mat = Arc::new(Lambertian::new(Vec3::new(0.9, 0.1, 0.1)));
    let mat = Arc::new(Normals());
    let metal = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.));
    let mesh_1 = Instance::from_trs(
        mesh.clone(),
        Vec3::new(-1.5, 0., 0.),
        Quat::IDENTITY,
        Vec3::new(2.0, 2.0, 2.0),
    );
    let mesh_2 = Instance::from_t(mesh.clone(), Vec3::new(5.5, 0., 0.));
    let sphere_mesh = Instance::from_t(Arc::new(sphere), Vec3::new(-10.5, 0., 0.));
    world.objs.push(mesh_1.with_mat(mat.clone()));
    world.objs.push(mesh_2.with_mat(metal.clone()));
    world.objs.push(sphere_mesh.with_mat(metal));
    world.build();
    for obj in &world.objs {
        dbg!(obj.node_index);
    }

    world.render(
        "two_spheres.png",
        height,
        origin,
        lookat,
        vfov,
        Vec3::new(0.7, 0.8, 1.),
        16. / 9.,
    );
}

fn render_random_spheres() {
    println!("Setup");
    let height = 480;
    let world = random_sphere_world();
    let vfov = 20.;

    let origin = Vec3::new(13., 2., 3.);
    let lookat = Vec3::new(0., 0., 0.);

    world.render(
        "random_spheres.png",
        height,
        origin,
        lookat,
        vfov,
        Vec3::new(0.7, 0.8, 1.),
        16. / 9.,
    );
}

fn render_cubes() {
    println!("Setup");
    let height = 720;
    let origin = Vec3::new(0., 7., -26.);
    let lookat = Vec3::new(0., 2., 0.);
    let vfov = 20.;
    let mut world = World::new(vec![]);
    let cube = Arc::new(Mesh::from_file("cube.obj", false));

    let mat_ground = Arc::new(Lambertian::from_tex(Arc::new(CheckerTex::from_colors(
        Vec3::new(0.2, 0.3, 0.1),
        Vec3::new(0.9, 0.9, 0.9),
    ))));
    let ground = Sphere::new(Vec3::new(0., -1000.5, -1.), 1000.);

    world.objs.push(ground.with_mat(mat_ground));

    // let sphere = Sphere::new(Vec3::ZERO, 1.0);
    // let mat = Arc::new(Lambertian::new(Vec3::new(0.9, 0.1, 0.1)));
    let mat = Arc::new(Normals());
    let metal = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.));
    let light = Arc::new(DiffuseLight::new(Vec3::new(4., 4., 4.)));

    let mesh_1 = Instance::from_trs(
        cube.clone(),
        Vec3::new(0., 0., 0.),
        // Quat::from_axis_angle(Vec3::Y, (-45. as f32).to_radians()),
        Quat::IDENTITY,
        Vec3::new(2.0, 2.0, 2.0),
    );
    let light_cube = Instance::from_trs(
        cube.clone(),
        Vec3::new(3., 0., 0.),
        Quat::from_axis_angle(Vec3::Z, (-45. as f32).to_radians()),
        Vec3::new(1.0, 1., 1.),
    );
    let sphere_mesh = Instance::from_t(cube, Vec3::new(-3., 0., 0.));
    world.objs.push(mesh_1.with_mat(mat.clone()));
    //world.objs.push(light_cube.with_mat(light));
    // world.objs.push(sphere_mesh.with_mat(metal));
    world.build();

    world.render(
        "cubes.png",
        height,
        origin,
        lookat,
        vfov,
        Vec3::new(0.7, 0.8, 1.),
        16. / 9.,
    );
}

fn cornell_box() {
    println!("Setup");
    let height = 480;
    let origin = Vec3::new(278., 278., -800.);
    let lookat = Vec3::new(278., 278., 0.);
    let vfov = 40.;
    let mut world = World::new(vec![]);
    let cube = Arc::new(Mesh::from_file("cube.obj", false));

    // let sphere = Sphere::new(Vec3::ZERO, 1.0);
    // let mat = Arc::new(Lambertian::new(Vec3::new(0.9, 0.1, 0.1)));
    let red = Arc::new(Lambertian::new(Vec3::new(0.65, 0.1, 0.1)));
    let white = Arc::new(Lambertian::new(Vec3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Vec3::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Vec3::new(15., 15., 15.)));

    let left = Instance::from_trs(
        cube.clone(),
        Vec3::new(0., 0., 0.),
        Quat::IDENTITY,
        Vec3::new(0.1, 555., 555.),
    );
    let right = Instance::from_trs(
        cube.clone(),
        Vec3::new(555., 0., 0.),
        Quat::IDENTITY,
        Vec3::new(0.1, 555., 555.),
    );
    let back = Instance::from_trs(
        cube.clone(),
        Vec3::new(0., 0., 555.),
        Quat::IDENTITY,
        Vec3::new(555., 555., 0.1),
    );
    let top = Instance::from_trs(
        cube.clone(),
        Vec3::new(0., 555., 0.),
        Quat::IDENTITY,
        Vec3::new(555., 0.1, 555.),
    );
    let bottom = Instance::from_trs(
        cube.clone(),
        Vec3::new(0., 0., 0.),
        Quat::IDENTITY,
        Vec3::new(555., 0.1, 555.),
    );
    let light_cube = Instance::from_trs(
        cube.clone(),
        Vec3::new(213., 554., 227.),
        Quat::IDENTITY,
        Vec3::new(130., 0.01, 107.),
    );

    
    let front_cube = Instance::from_trs(
        cube.clone(),
        Vec3::new(130., 0., 65.),
        Quat::from_axis_angle(Vec3::Y, (-18. as f32).to_radians()),
        Vec3::new(165., 165., 165.),
    );

    
    let back_cube = Instance::from_trs(
        cube.clone(),
        Vec3::new(265., 0., 195.),
        Quat::from_axis_angle(Vec3::Y, (15. as f32).to_radians()),
        Vec3::new(165., 330., 165.),
    );
    dbg!(left.aabb());
    world.objs.push(left.with_mat(red.clone()));
    world.objs.push(right.with_mat(green.clone()));
    world.objs.push(back.with_mat(white.clone()));
    world.objs.push(top.with_mat(white.clone()));
    world.objs.push(bottom.with_mat(white.clone()));
    world.objs.push(light_cube.with_mat(light));
    world.objs.push(front_cube.with_mat(white.clone()));
    world.objs.push(back_cube.with_mat(white.clone()));
    world.build();

    world.render(
        "cornell_box.png",
        height,
        origin,
        lookat,
        vfov,
        Vec3::new(0., 0., 0.),
        1.,
    );
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
