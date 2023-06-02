pub mod plane;
pub mod sphere;

pub enum Element {
    Plane(plane::Plane),
    Sphere(plane::Plane),
}

impl Element {
    pub fn draw() {}
}
