use bvh::ray::{IntersectionRay, Intersection, Ray};
use glam::Vec3;

use crate::{color::Color, rand_unit_vector, reflect};

pub trait Material: Sync {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)>;

}

#[derive(Clone, Copy)]
pub struct WithMat<'a, 'b> {
    pub obj: &'a (dyn IntersectionRay + Sync),
    pub mat: &'b (dyn Material),
}

impl<'a, 'b> WithMat<'a, 'b>{
    pub fn new(obj: &'a (dyn IntersectionRay + Sync), mat: &'b (dyn Material)) -> Self {
        Self { obj, mat }
    }
}

impl<'a, 'b> Material for WithMat<'a, 'b> {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        self.mat.scatter(ray, intersection)
    }
}

impl<'a, 'b> IntersectionRay for WithMat<'a, 'b> {
    fn intersects_ray(&self, ray: &Ray, t_min: bvh::Real, t_max: bvh::Real) -> Option<Intersection> {
        self.obj.intersects_ray(ray, t_min, t_max)
    }
}

pub trait ToWithMat {
    fn with_mat<'a, 'b>(&'a self, mat: &'b (dyn Material)) -> WithMat<'a, 'b>;
}

impl <T> ToWithMat for T where T : IntersectionRay + Sync {
    fn with_mat<'a, 'b>(&'a self, mat: &'b (dyn Material)) -> WithMat<'a, 'b> {
        WithMat::new(self, mat)
    }
}



pub struct BoxMat<'b, O: IntersectionRay + Sync + ?Sized, M: Material + ?Sized> {
    pub obj: Box<O>,
    pub mat: &'b M,
}


impl<'b, O: IntersectionRay + Sync, M: Material> BoxMat<'b, O, M>{
    pub fn new(obj: O, mat: &'b M) -> Self {
        Self { obj: Box::new(obj), mat }
    }
}

impl<'b, O, M> Material for BoxMat<'b, O, M> where O: IntersectionRay + Sync, M: Material {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        self.mat.scatter(ray, intersection)
    }
}

impl<'b, O, M> IntersectionRay for BoxMat<'b, O, M> where O: IntersectionRay + Sync, M: Material {
    fn intersects_ray(&self, ray: &Ray, t_min: bvh::Real, t_max: bvh::Real) -> Option<Intersection> {
        self.obj.intersects_ray(ray, t_min, t_max)
    }
}




pub struct Lambertian {
    pub albedo: Vec3
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self {
            albedo
        }
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
    pub albedo: Vec3
}

impl Metal {
    pub fn new(albedo: Vec3) -> Self {
        Self {
            albedo
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        let reflected = reflect(ray.direction, intersection.norm);
        if reflected.dot(intersection.norm) > 0. {
            Some((Ray::new(ray.at(intersection.distance), reflected), self.albedo))
        } else {
            None
        }
    }
}



