use std::{fs::File, io::BufReader};

use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    bvh::BVH,
    ray::{Intersection, IntersectionRay, Ray},
    Real, Triangle,
};
use itertools::Itertools;
use obj::{load_obj, Obj, Position};

pub struct Mesh {
    pub triangles: Vec<Indexed<Triangle>>,
    bvh: BVH,
}

pub struct Indexed<T>
where
    T: Bounded + IntersectionRay,
{
    pub obj: T,
    pub shape_idx: usize,
}

impl<T> Indexed<T>
where
    T: Bounded + IntersectionRay + Sync + Send,
{
    pub fn new(obj: T) -> Self {
        Self { obj, shape_idx: 0 }
    }
}

impl<T> Bounded for Indexed<T>
where
    T: Bounded + IntersectionRay + Sync + Send,
{
    fn aabb(&self) -> AABB {
        self.obj.aabb()
    }
}

impl<T> BHShape for Indexed<T>
where
    T: Bounded + IntersectionRay + Sync + Send,
{
    fn set_bh_node_index(&mut self, idx: usize) {
        self.shape_idx = idx;
    }

    fn bh_node_index(&self) -> usize {
        self.shape_idx
    }
}
impl<T> IntersectionRay for Indexed<T>
where
    T: Bounded + IntersectionRay + Sync + Send,
{
    fn intersects_ray(&self, ray: &Ray, t_min: Real, t_max: Real) -> Option<Intersection> {
        self.obj.intersects_ray(ray, t_min, t_max)
    }
}

impl Mesh {
    pub fn new() -> Self {
        let mut triangles = vec![];
        let bvh = BVH::build(&mut triangles);
        Self { triangles, bvh }
    }

    pub fn from_file(path: &str) -> Self {
        let mut mesh = Mesh::new();
        let input = BufReader::new(File::open(path).unwrap());
        println!("Loading");
        let obj: Obj<Position> = load_obj(input).unwrap();
        println!("done");
        mesh.triangles = obj
            .indices
            .iter()
            .tuples()
            .map(|(&a, &b, &c)| {
                let vert_a = obj.vertices[a as usize];
                let vert_b = obj.vertices[b as usize];
                let vert_c = obj.vertices[c as usize];
                let tri = Triangle::new(
                    vert_a.position.into(),
                    vert_b.position.into(),
                    vert_c.position.into(),
                );
                Indexed::new(tri)
            })
            .collect();

        mesh.rebuild();

        mesh
    }

    pub fn rebuild(&mut self) {
        self.bvh.rebuild(&mut self.triangles)
    }
}

impl IntersectionRay for Mesh {
    fn intersects_ray(
        &self,
        ray: &bvh::ray::Ray,
        t_min: bvh::Real,
        t_max: bvh::Real,
    ) -> Option<bvh::ray::Intersection> {
        self.bvh
            .traverse_iterator(ray, &self.triangles)
            .fold(None, |hit, tri| {
                if let Some(inter) = tri.intersects_ray(ray, t_min, t_max) {
                    if let Some(last_inter) = hit {
                        if inter.distance < last_inter.distance {
                            Some(inter)
                        } else {
                            Some(last_inter)
                        }
                    } else {
                        Some(inter)
                    }
                } else {
                    hit
                }
            })
    }
}

impl Bounded for Mesh {
    fn aabb(&self) -> AABB {
        if self.triangles.len() == 0 {
            return AABB::empty();
        }

        self.bvh.nodes[0].get_node_aabb(&self.triangles)
    }
}
