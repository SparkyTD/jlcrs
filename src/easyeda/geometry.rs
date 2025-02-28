use serde::{Deserialize, Serialize};
use crate::kicad::model::footprint_library::Scalar2D;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Point2D {
        Point2D { x, y }
    }

    pub fn to_scalar_2d(&self, identifier: &str) -> Scalar2D {
        Scalar2D::new(identifier, self.x, self.y)
    }
}