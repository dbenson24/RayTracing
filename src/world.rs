use std::time::Instant;

use bvh::{
    aabb::Bounded,
    bvh::BVH,
    ray::{Intersection, IntersectionRay, Ray},
};
use glam::Vec3;
use image::ImageBuffer;

use crate::{
    camera::Camera,
    color::{Color, RGB},
    material::{Material, WithMat},
    random,
};
use rayon::prelude::*;
pub trait Hittable: IntersectionRay + Bounded + Sync + Send {}

impl<T> Hittable for T where T: IntersectionRay + Bounded + Sync + Send {}

pub struct World {
    pub objs: Vec<WithMat>,
    bvh: BVH,
}

impl World {
    pub fn new(mut objs: Vec<WithMat>) -> Self {
        let bvh = BVH::build(&mut objs);
        World { objs, bvh }
    }

    pub fn build(&mut self) {
        self.bvh.rebuild(&mut self.objs)
    }

    pub fn first_intersection<'a>(
        &'a self,
        ray: Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<(&'a WithMat, Intersection)> {
        // self.bvh.traverse_best_first(t_min, t_max, |aabb| {
        //     ray.intersects_aabb_dist(aabb)
        // }, |obj_idx| {
        //     let obj = &self.objs[obj_idx];
        //     if let Some(inter) = obj.intersects_ray(&ray, t_min, t_max) {
        //         Some((inter.distance, (obj, inter)))
        //     } else {
        //         None
        //     }
        // })
        self.bvh
            .traverse_iterator(&ray, &self.objs)
            .fold(None, |hit, obj| {
                if let Some(inter) = obj.intersects_ray(&ray, t_min, t_max) {
                    if let Some((last_obj, last_inter)) = hit {
                        if inter.distance < last_inter.distance {
                            Some((obj, inter))
                        } else {
                            Some((last_obj, last_inter))
                        }
                    } else {
                        Some((obj, inter))
                    }
                } else {
                    hit
                }
            })
    }

    pub fn render(
        &self,
        path: &str,
        height: usize,
        origin: Vec3,
        lookat: Vec3,
        vfov: f32,
        background: Color,
        aspect_ratio: f32,
    ) {
        let width = (height as f32 * aspect_ratio) as usize;
        let mut pixels = vec![Color::default(); width * height];

        let samples_per_px = 200;
        let max_bounces = 50;
        let aperture = 0.01;

        // Camera
        // let dist_to_focus = (lookat - origin).length();
        let dist_to_focus = 10.;
        let up = Vec3::new(0., 1., 0.);
        let camera = Camera::new(
            origin,
            lookat,
            up,
            vfov,
            aspect_ratio,
            aperture,
            dist_to_focus,
        );

        println!("Begin Tracing");

        let now = Instant::now();
        pixels.par_iter_mut().enumerate().for_each(|(i, px)| {
            let x = i % width;
            let y = (height - 1) - (i / width);
            for _ in 0..samples_per_px {
                let u = (x as f32 + random()) / (width - 1) as f32;
                let v = (y as f32 + random()) / (height - 1) as f32;
                let ray = camera.get_ray(u, v);
                *px += self.ray_color(&ray, max_bounces, background);
            }
        });

        let elapsed = now.elapsed();
        println!("Done Tracing in {} ms", elapsed.as_millis());

        let image = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            let i = (x + (y * width as u32)) as usize;
            let c = pixels[i];
            c.to_px(samples_per_px)
        });

        image.save(path).expect("Image to save");
        println!("Image written to {}", path);
    }

    pub fn ray_color(&self, ray: &Ray, depth: usize, background: Color) -> Color {
        if depth == 0 {
            return Vec3::ZERO;
        }

        if let Some((obj, intersection)) = self.first_intersection(*ray, 0.001, f32::INFINITY) {
            let emit = obj.emit(
                intersection.u,
                intersection.v,
                &ray.at(intersection.distance),
            );
            if let Some((child_ray, attenuation, pdf)) = obj.scatter(ray, &intersection) {
                emit + self.ray_color(&child_ray, depth - 1, background) * attenuation * obj.scattering_pdf(&ray, &intersection, &child_ray) / pdf
            } else {
                emit
            }
        } else {
            background
        }
    }
}
