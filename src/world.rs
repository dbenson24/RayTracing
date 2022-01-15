use bvh::ray::{Intersection, IntersectionRay, Ray};

use crate::material::Material;

pub trait Hittable : IntersectionRay + Material {}

impl <T> Hittable for T where T : IntersectionRay + Material {}

pub struct World<> {
    pub objs: Vec<Box<dyn Hittable + Sync>>,
}

impl World {
    pub fn new(objs: Vec<Box<dyn Hittable + Sync>>) -> Self {
        World { objs }
    }
}

impl World {
    pub fn first_intersection(
        &self,
        ray: &Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<(&Box<dyn Hittable + Sync>, Intersection)> {
        self.objs.iter().fold(None, |hit, obj| {
            if let Some(inter) = obj.intersects_ray(ray, t_min, t_max) {
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
}
