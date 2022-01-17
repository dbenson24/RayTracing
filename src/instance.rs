use std::sync::Arc;

use bvh::{
    aabb::{Bounded, AABB},
    ray::{Intersection, IntersectionRay, Ray},
};
use glam::{Mat4, Quat, Vec3};

pub struct Instance<T> {
    transform: Mat4,
    obj: Arc<T>,
}

impl<T> Instance<T> {
    pub fn new(obj: Arc<T>, transform: Mat4) -> Self {
        Self { obj, transform }
    }

    pub fn from_trs(obj: Arc<T>, translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        Self { obj, transform }
    }

    pub fn from_tr(obj: Arc<T>, translation: Vec3, rotation: Quat) -> Self {
        let transform = Mat4::from_scale_rotation_translation(Vec3::ONE, rotation, translation);
        Self { obj, transform }
    }

    pub fn from_t(obj: Arc<T>, translation: Vec3) -> Self {
        let transform =
            Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, translation);
        Self { obj, transform }
    }
}

impl<T> IntersectionRay for Instance<T>
where
    T: IntersectionRay,
{
    fn intersects_ray(
        &self,
        ray: &Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<Intersection> {
        let inv = self.transform.inverse();
        let local_ray = Ray::new(
            inv.transform_point3(ray.origin),
            inv.transform_vector3(ray.direction).normalize(),
        );
        //dbg!(ray.origin, local_ray.origin);
        if let Some(intersection) = self.obj.intersects_ray(&local_ray, t_min, t_max) {
            let hit_pos = local_ray.at(intersection.distance);
            let world_hit = self.transform.transform_point3(hit_pos);

            let distance = (world_hit - ray.origin).length();

            let norm = self.transform.transform_vector3(intersection.norm);
            let u = intersection.u;
            let v = intersection.v;
            let back_face = intersection.back_face;
            Some(Intersection::new(distance, u, v, norm, back_face))
        } else {
            None
        }
    }
}

impl<T> Bounded for Instance<T>
where
    T: Bounded,
{
    fn aabb(&self) -> AABB {
        let aabb = self.obj.aabb();
        let min = self.transform.transform_point3(aabb.min);
        let max = self.transform.transform_point3(aabb.max);
        //dbg!(aabb);
        //dbg!(min, max);
        let mut bounds = AABB::empty();
        bounds.grow_mut(&min);
        bounds.grow_mut(&max);
        bounds
    }
}
