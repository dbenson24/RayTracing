use std::{fs::File, io::BufReader, rc::Rc, sync::Arc};

use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    bvh::BVH,
    ray::{Intersection, IntersectionRay, Ray},
    Real, Triangle,
};
use glam::Vec3;
use itertools::Itertools;
use obj::{load_obj, Obj, Position};

pub struct Mesh {
    pub triangles: Vec<Indexed<RefTri>>,
    pub vertices: Arc<Vec<Vec3>>,
    pub normals: Arc<Vec<Vec3>>,
    bvh: BVH,
}

pub struct RefTri {
    pub verts: Arc<Vec<Vec3>>,
    pub norms: Arc<Vec<Vec3>>,
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub smooth: bool,
}

impl RefTri {
    pub fn a_pos(&self) -> Vec3 {
        self.verts[self.a as usize]
    }
    pub fn b_pos(&self) -> Vec3 {
        self.verts[self.b as usize]
    }
    pub fn c_pos(&self) -> Vec3 {
        self.verts[self.c as usize]
    }
    pub fn a_norm(&self) -> Vec3 {
        self.norms[self.a as usize]
    }
    pub fn b_norm(&self) -> Vec3 {
        self.norms[self.b as usize]
    }
    pub fn c_norm(&self) -> Vec3 {
        self.norms[self.c as usize]
    }
}

impl Bounded for RefTri {
    fn aabb(&self) -> AABB {
        AABB::empty()
            .grow(&self.a_pos())
            .grow(&self.b_pos())
            .grow(&self.c_pos())
    }
}

impl IntersectionRay for RefTri {
    fn intersects_ray(&self, ray: &Ray, t_min: Real, t_max: Real) -> Option<Intersection> {
        let mut inter = ray.intersects_triangle(&self.a_pos(), &self.b_pos(), &self.c_pos());
        if inter.distance < f32::INFINITY {
            //let old_norm = inter.norm;
            // dbg!(inter.u, inter.v);
            if self.smooth {
                inter.norm = (inter.u * self.b_norm())
                    + (inter.v * self.c_norm())
                    + ((1. - inter.u - inter.v) * self.a_norm());
            }
            //inter.norm = (0.333333 * self.a_norm()) + (0.333333 * self.b_norm()) + (0.333333 * self.c_norm());
            // inter.norm = Vec3::new(inter.u, inter.v, 1. - inter.u - inter.v);
            // dbg!(inter.norm, old_norm);
            Some(inter)
        } else {
            None
        }
    }
}

impl Mesh {
    pub fn new() -> Self {
        let mut triangles = vec![];
        let bvh = BVH::build(&mut triangles);
        Self {
            triangles,
            bvh,
            normals: Arc::new(vec![]),
            vertices: Arc::new(vec![]),
        }
    }

    pub fn from_file(path: &str, smooth: bool) -> Self {
        let mut mesh = Mesh::new();
        let input = BufReader::new(File::open(path).unwrap());
        println!("Loading");
        let obj: Obj<Position> = load_obj(input).unwrap();
        println!("done");
        let vertices: Vec<Vec3> = obj.vertices.iter().map(|x| x.position.into()).collect();
        let mut normals = vec![Vec3::ZERO; vertices.len()];
        let mut nums: Vec<usize> = vec![0; vertices.len()];
        for (&a, &b, &c) in obj.indices.iter().tuples() {
            let a = a as usize;
            let b = b as usize;
            let c = c as usize;
            nums[a] += 1;
            nums[b] += 1;
            nums[c] += 1;

            let a_to_b = vertices[b] - vertices[a];
            let a_to_c = vertices[c] - vertices[a];
            let mut normal = Vec3::ZERO;
            normal.x = (a_to_b.y * a_to_c.z) - (a_to_b.z * a_to_c.y);
            normal.y = (a_to_b.z * a_to_c.x) - (a_to_b.x * a_to_c.z);
            normal.z = (a_to_b.x * a_to_c.y) - (a_to_b.y * a_to_c.x);
            normal = normal.normalize();

            normals[a] += normal;
            normals[b] += normal;
            normals[c] += normal;
        }

        normals
            .iter_mut()
            .zip(nums.iter())
            .for_each(|(norm, count)| {
                *norm = (*norm / (*count as f32)).normalize();
            });

        mesh.normals = Arc::new(normals);
        mesh.vertices = Arc::new(vertices);

        mesh.triangles = obj
            .indices
            .iter()
            .tuples()
            .map(|(&a, &b, &c)| {
                let tri = RefTri {
                    a,
                    b,
                    c,
                    verts: mesh.vertices.clone(),
                    norms: mesh.normals.clone(),
                    smooth,
                };
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
