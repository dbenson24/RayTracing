use bvh::{ray::{Intersection, IntersectionRay, Ray}, bounding_hierarchy::BHShape, aabb::{Bounded, AABB}};
use glam::Vec3;

use crate::{
    color::Color, rand_in_sphere, rand_unit_vector, random, reflect, reflectance, refract, world::Hittable,
};

pub trait Material: Sync {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)>;
}

#[derive(Clone, Copy)]
pub struct WithMat<'a, 'b> {
    pub obj: &'a (dyn Hittable + Sync),
    pub mat: &'b (dyn Material),
    node_index: usize
}

impl<'a, 'b> WithMat<'a, 'b> {
    pub fn new(obj: &'a (dyn Hittable + Sync), mat: &'b (dyn Material)) -> Self {
        Self { obj, mat, node_index: 0 }
    }
}

impl<'a, 'b> Material for WithMat<'a, 'b> {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        self.mat.scatter(ray, intersection)
    }
}

impl<'a, 'b> IntersectionRay for WithMat<'a, 'b> {
    fn intersects_ray(
        &self,
        ray: &Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<Intersection> {
        self.obj.intersects_ray(ray, t_min, t_max)
    }
}

impl BHShape for WithMat<'_, '_> {
    fn set_bh_node_index(&mut self, idx: usize) {
        self.node_index = idx
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

impl Bounded for WithMat<'_, '_> {
    fn aabb(&self) -> AABB {
        self.obj.aabb()
    }
}

pub trait ToWithMat {
    fn with_mat<'a, 'b>(&'a self, mat: &'b (dyn Material)) -> WithMat<'a, 'b>;
}

impl<T> ToWithMat for T
where
    T: Hittable + Sync,
{
    fn with_mat<'a, 'b>(&'a self, mat: &'b (dyn Material)) -> WithMat<'a, 'b> {
        WithMat::new(self, mat)
    }
}

pub struct Lambertian {
    pub albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        let mut scatter_direction = intersection.norm + rand_unit_vector();

        if scatter_direction.abs().min_element() < 1e-6 {
            scatter_direction = intersection.norm
        }

        let ray = Ray::new(ray.at(intersection.distance), scatter_direction);
        Some((ray, self.albedo))
    }
}

pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f32) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        let reflected = reflect(ray.direction, intersection.norm) + (self.fuzz * rand_in_sphere());
        if reflected.dot(intersection.norm) > 0. {
            Some((
                Ray::new(ray.at(intersection.distance), reflected),
                self.albedo,
            ))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    pub index_of_refraction: f32,
}

impl Dielectric {
    pub fn new(index_of_refraction: f32) -> Self {
        Self {
            index_of_refraction,
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if intersection.back_face {
            self.index_of_refraction
        } else {
            1.0 / self.index_of_refraction
        };
        let cos_theta = (-ray.direction).dot(intersection.norm).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();
        let direction = if refraction_ratio * sin_theta > 1.
            || reflectance(cos_theta, refraction_ratio) > random()
        {
            reflect(ray.direction, intersection.norm)
        } else {
            refract(ray.direction, intersection.norm, refraction_ratio)
        };

        Some((
            Ray::new(ray.at(intersection.distance), direction),
            attenuation,
        ))
    }
}
