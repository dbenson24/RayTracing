use bvh::{ray::{Intersection, IntersectionRay, Ray}, aabb::Bounded, bvh::BVH};

use crate::material::{Material, WithMat};

pub trait Hittable: IntersectionRay + Bounded {}

impl<T> Hittable for T where T: IntersectionRay + Bounded {}

pub struct World<'a, 'b> {
    pub objs: Vec<WithMat<'a, 'b>>,
    bvh: BVH
}

impl<'a, 'b> World<'a, 'b> {
    pub fn new(mut objs: Vec<WithMat<'a, 'b>>) -> Self {
        let bvh = BVH::build(&mut objs);
        World { objs, bvh}
    }

    pub fn build(&mut self) {
        self.bvh.rebuild(&mut self.objs)
    }

    pub fn first_intersection(
        &self,
        ray: &Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<(WithMat<'a, 'b>, Intersection)> {
        self.bvh.traverse_iterator(ray, &self.objs).fold(None, |hit, obj| {
            if let Some(inter) = obj.intersects_ray(ray, t_min, t_max) {
                if let Some((last_obj, last_inter)) = hit {
                    if inter.distance < last_inter.distance {
                        Some((*obj, inter))
                    } else {
                        Some((last_obj, last_inter))
                    }
                } else {
                    Some((*obj, inter))
                }
            } else {
                hit
            }
        })
    }
}
