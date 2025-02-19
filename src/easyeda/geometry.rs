use serde_json::Value;

pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

pub enum Geometry {
    Polygon { points: Vec<Point2D> },
    Rectangle { start: Point2D, width: f32, height: f32, rotation: f32, corner_radius: f32 },
    Circle { center: Point2D, radius: f32 },
    Arc { start: Point2D, end: Point2D, rotation: f32, center_arc: bool },
}

enum GeometryType {
    Polygon,
    Rectangle,
    Circle,
    Arc,
    CenterArc,
}

pub struct GeometryParser {
    raw_value: Vec<Value>,
    read_index: usize,

    value_stack: Vec<Value>,
    current_geometry: Option<Geometry>,
}

impl GeometryParser {
    pub fn new(mut values: Vec<Value>) -> Self {
        if values.get(0).unwrap().is_array() {
            values = values.iter()
                .map(|v| v.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.clone()))
                .flatten()
                .collect();
        }
        Self { raw_value: values, read_index: 0, value_stack: Vec::new(), current_geometry: None }
    }
}
