use crate::kicad::model::footprint_library::{PcbLayer, Scalar2D};
use crate::kicad::syntax::{PositionPreference, SyntaxArgument, SyntaxItem, SyntaxItemSerializable};

#[derive(Debug)]
pub struct GraphicLine {
    pub start: Scalar2D,
    pub end: Scalar2D,
    pub angle: Option<f32>,
    pub layer: Option<PcbLayer>,
    pub width: f32,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub struct GraphicPolygon {
    pub points: Vec<Scalar2D>,
    pub layer: Option<PcbLayer>,
    pub width: Option<f32>,
    pub fill: Option<bool>,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub struct GraphicRectangle {
    pub start: Scalar2D,
    pub end: Scalar2D,
    pub layer: Option<PcbLayer>,
    pub width: f32,
    pub fill: Option<bool>,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub struct GraphicCircle {
    pub center: Scalar2D,
    pub end: Scalar2D,
    pub layer: Option<PcbLayer>,
    pub width: f32,
    pub fill: Option<bool>,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub struct GraphicArc {
    pub start: Scalar2D,
    pub mid: Scalar2D,
    pub end: Scalar2D,
    pub layer: Option<PcbLayer>,
    pub width: f32,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub struct GraphicCurve {
    pub points: Vec<Scalar2D>,
    pub layer: Option<PcbLayer>,
    pub width: f32,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub struct GraphicAnnotationBox {
    pub start: Scalar2D,
    pub end: Scalar2D,
}

impl SyntaxItemSerializable for GraphicLine {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.start.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("width", SyntaxArgument::Number(self.width, PositionPreference::None)),
        ];

        if let Some(angle) = self.angle {
            children.push(SyntaxItem::from_single_argument("angle", SyntaxArgument::Number(angle, PositionPreference::None)));
        }
        if let Some(layers) = &self.layer {
            children.push(layers.serialize());
        }
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::Identifier(uuid.clone(), PositionPreference::None)));
        }

        SyntaxItem {
            name: "gr_line".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut line = Self {
            start: Scalar2D::default(),
            end: Scalar2D::default(),
            angle: None,
            layer: None,
            width: 0.0,
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "start" => line.start = Scalar2D::deserialize(child),
                "end" => line.end = Scalar2D::deserialize(child),
                "width" => line.width = child.arguments.get(0).unwrap().get_number(),
                "angle" => line.angle = Some(child.arguments.get(0).unwrap().get_number()),
                "layers" => line.layer = Some(PcbLayer::deserialize(child)),
                "uuid" => line.uuid = Some(child.arguments.first().unwrap().get_string()),
                _ => panic!("Unsupported child item type in GraphicLine: {}", child.name),
            }
        }

        line
    }
}

impl SyntaxItemSerializable for GraphicPolygon {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            // Points are wrapped in a "pts" node
            SyntaxItem {
                name: "pts".into(),
                arguments: vec![],
                children: self.points.iter().map(|point| point.serialize()).collect(),
            },
        ];

        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(width, PositionPreference::None)));
        }

        // Add optional layers if present
        if let Some(layers) = &self.layer {
            children.push(layers.serialize());
        }

        // Add optional fill if present
        if let Some(fill) = self.fill {
            children.push(SyntaxItem::from_single_argument(
                "fill",
                SyntaxArgument::Identifier(
                    if fill { "solid" } else { "none" }.into(),
                    PositionPreference::None
                )
            ));
        }

        // Add optional UUID if present
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument(
                "uuid",
                SyntaxArgument::Identifier(uuid.clone(), PositionPreference::None)
            ));
        }

        SyntaxItem {
            name: "gr_poly".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut poly = Self {
            points: Vec::new(),
            layer: None,
            width: None,
            fill: None,
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "pts" => {
                    poly.points = child.children.iter()
                        .map(|c| Scalar2D::deserialize(c))
                        .collect();
                }
                "width" => {
                    poly.width = Some(child.arguments.get(0).unwrap().get_number());
                }
                "layers" => {
                    poly.layer = Some(PcbLayer::deserialize(child));
                }
                "fill" => {
                    let fill_type = child.arguments.first().unwrap().get_string();
                    poly.fill = Some(fill_type == "solid" || fill_type == "yes");
                }
                "uuid" => {
                    poly.uuid = Some(child.arguments.first().unwrap().get_string());
                }
                _ => panic!("Unsupported child item type in GraphicPolygon: {}", child.name),
            }
        }

        poly
    }
}

impl SyntaxItemSerializable for GraphicRectangle {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.start.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("width", SyntaxArgument::Number(self.width, PositionPreference::None)),
        ];

        // Add optional layers if present
        if let Some(layers) = &self.layer {
            children.push(layers.serialize());
        }

        // Add optional fill if present
        if let Some(fill) = self.fill {
            children.push(SyntaxItem::from_single_argument(
                "fill",
                SyntaxArgument::Identifier(
                    if fill { "solid" } else { "none" }.into(),
                    PositionPreference::None
                )
            ));
        }

        // Add optional UUID if present
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument(
                "uuid",
                SyntaxArgument::Identifier(uuid.clone(), PositionPreference::None)
            ));
        }

        SyntaxItem {
            name: "gr_rect".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut rect = Self {
            start: Scalar2D::default(),
            end: Scalar2D::default(),
            layer: None,
            width: 0.0,
            fill: None,
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "start" => rect.start = Scalar2D::deserialize(child),
                "end" => rect.end = Scalar2D::deserialize(child),
                "width" => rect.width = child.arguments.get(0).unwrap().get_number(),
                "layers" => rect.layer = Some(PcbLayer::deserialize(child)),
                "fill" => {
                    let fill_type = child.arguments.first().unwrap().get_string();
                    rect.fill = Some(fill_type == "solid");
                }
                "uuid" => rect.uuid = Some(child.arguments.first().unwrap().get_string()),
                _ => panic!("Unsupported child item type in GraphicRectangle: {}", child.name),
            }
        }

        rect
    }
}

impl SyntaxItemSerializable for GraphicCircle {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.center.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("width", SyntaxArgument::Number(self.width, PositionPreference::None)),
        ];

        // Add optional layers if present
        if let Some(layers) = &self.layer {
            children.push(layers.serialize());
        }

        // Add optional fill if present
        if let Some(fill) = self.fill {
            children.push(SyntaxItem::from_single_argument(
                "fill",
                SyntaxArgument::Identifier(
                    if fill { "solid" } else { "none" }.into(),
                    PositionPreference::None
                )
            ));
        }

        // Add optional UUID if present
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument(
                "uuid",
                SyntaxArgument::Identifier(uuid.clone(), PositionPreference::None)
            ));
        }

        SyntaxItem {
            name: "gr_circle".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut circle = Self {
            center: Scalar2D::default(),
            end: Scalar2D::default(),
            layer: None,
            width: 0.0,
            fill: None,
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "center" => circle.center = Scalar2D::deserialize(child),
                "end" => circle.end = Scalar2D::deserialize(child),
                "width" => circle.width = child.arguments.get(0).unwrap().get_number(),
                "layers" => circle.layer = Some(PcbLayer::deserialize(child)),
                "fill" => {
                    let fill_type = child.arguments.first().unwrap().get_string();
                    circle.fill = Some(fill_type == "solid");
                }
                "uuid" => circle.uuid = Some(child.arguments.first().unwrap().get_string()),
                _ => panic!("Unsupported child item type in GraphicCircle: {}", child.name),
            }
        }

        circle
    }
}

impl SyntaxItemSerializable for GraphicArc {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.start.serialize(),
            self.mid.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("width", SyntaxArgument::Number(self.width, PositionPreference::None)),
        ];

        // Add optional layers if present
        if let Some(layers) = &self.layer {
            children.push(layers.serialize());
        }

        // Add optional UUID if present
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument(
                "uuid",
                SyntaxArgument::Identifier(uuid.clone(), PositionPreference::None)
            ));
        }

        SyntaxItem {
            name: "gr_arc".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut arc = Self {
            start: Scalar2D::default(),
            mid: Scalar2D::default(),
            end: Scalar2D::default(),
            layer: None,
            width: 0.0,
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "start" => arc.start = Scalar2D::deserialize(child),
                "mid" => arc.mid = Scalar2D::deserialize(child),
                "end" => arc.end = Scalar2D::deserialize(child),
                "width" => arc.width = child.arguments.get(0).unwrap().get_number(),
                "layers" => arc.layer = Some(PcbLayer::deserialize(child)),
                "uuid" => arc.uuid = Some(child.arguments.first().unwrap().get_string()),
                _ => panic!("Unsupported child item type in GraphicArc: {}", child.name),
            }
        }

        arc
    }
}

impl SyntaxItemSerializable for GraphicCurve {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            // Points are wrapped in a "pts" node
            SyntaxItem {
                name: "pts".into(),
                arguments: vec![],
                children: self.points.iter().map(|point| point.serialize()).collect(),
            },
            SyntaxItem::from_single_argument("width", SyntaxArgument::Number(self.width, PositionPreference::None)),
        ];

        // Add optional layers if present
        if let Some(layers) = &self.layer {
            children.push(layers.serialize());
        }

        // Add optional UUID if present
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument(
                "uuid",
                SyntaxArgument::Identifier(uuid.clone(), PositionPreference::None)
            ));
        }

        SyntaxItem {
            name: "bezier".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut curve = Self {
            points: Vec::new(),
            layer: None,
            width: 0.0,
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "pts" => {
                    curve.points = child.children.iter()
                        .map(|c| Scalar2D::deserialize(c))
                        .collect();
                }
                "width" => {
                    curve.width = child.arguments.get(0).unwrap().get_number();
                }
                "layers" => {
                    curve.layer = Some(PcbLayer::deserialize(child));
                }
                "uuid" => {
                    curve.uuid = Some(child.arguments.first().unwrap().get_string());
                }
                _ => panic!("Unsupported child item type in GraphicCurve: {}", child.name),
            }
        }

        curve
    }
}

impl SyntaxItemSerializable for GraphicAnnotationBox {
    fn serialize(&self) -> SyntaxItem {
        let children = vec![
            self.start.serialize(),
            self.end.serialize(),
        ];

        SyntaxItem {
            name: "gr_bbox".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut box_annotation = Self {
            start: Scalar2D::default(),
            end: Scalar2D::default(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "start" => box_annotation.start = Scalar2D::deserialize(child),
                "end" => box_annotation.end = Scalar2D::deserialize(child),
                _ => panic!("Unsupported child item type in GraphicAnnotationBox: {}", child.name),
            }
        }

        box_annotation
    }
}