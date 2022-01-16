use std::{rc::Rc, sync::Arc};

use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    ray::{Intersection, IntersectionRay, Ray},
};
use glam::Vec3;

use crate::{
    color::Color,
    rand_in_sphere, rand_unit_vector, random, reflect, reflectance, refract,
    texture::{SolidTex, Texture},
    world::Hittable,
};

pub trait Material: Sync + Send {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)>;
}

#[derive(Clone)]
pub struct WithMat {
    pub obj: Arc<(dyn Hittable)>,
    pub mat: Arc<(dyn Material)>,
    pub node_index: usize,
}

impl WithMat {
    pub fn new(obj: Arc<(dyn Hittable)>, mat: Arc<(dyn Material)>) -> Self {
        Self {
            obj,
            mat,
            node_index: 0,
        }
    }
}

impl Material for WithMat {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        self.mat.scatter(ray, intersection)
    }
}

impl IntersectionRay for WithMat {
    fn intersects_ray(
        &self,
        ray: &Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<Intersection> {
        self.obj.intersects_ray(ray, t_min, t_max)
    }
}

impl BHShape for WithMat {
    fn set_bh_node_index(&mut self, idx: usize) {
        self.node_index = idx
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

impl Bounded for WithMat {
    fn aabb(&self) -> AABB {
        self.obj.aabb()
    }
}

pub trait ToWithMat {
    fn with_mat(self, mat: Arc<(dyn Material)>) -> WithMat;
}

impl<T> ToWithMat for T
where
    T: Hittable + Sync + 'static,
{
    fn with_mat(self, mat: Arc<(dyn Material)>) -> WithMat {
        WithMat::new(Arc::new(self), mat)
    }
}

pub struct Lambertian {
    pub albedo: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        let albedo = Arc::new(SolidTex::new(albedo));
        Self { albedo }
    }

    pub fn from_tex(albedo: Arc<dyn Texture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, intersection: &Intersection) -> Option<(Ray, Color)> {
        let mut scatter_direction = intersection.norm + rand_unit_vector();

        if scatter_direction.abs().min_element() < 1e-6 {
            scatter_direction = intersection.norm
        }
        let hit = ray.at(intersection.distance);

        let ray = Ray::new(hit, scatter_direction);
        Some((ray, self.albedo.value(intersection.u, intersection.v, &hit)))
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
