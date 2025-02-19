use crate::kicad::model::symbol_library::{Color, StrokeType};
use crate::kicad::syntax::{PositionPreference, SyntaxArgument, SyntaxItem, SyntaxItemSerializable};

#[derive(Debug, Clone)]
pub struct StrokeDefinition {
    pub width: f32,
    pub dash: Option<StrokeType>,
    pub color: Option<Color>,
}

#[derive(Debug, Default, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub angle: Option<f32>,
}

#[derive(Debug, Default, Clone)]
pub struct Id {
    pub id: u32,
}

#[derive(Debug, Default, Clone)]
pub struct TextEffect {
    pub font: Font,
    pub justify: TextJustify,
    pub hide: bool,
}

#[derive(Debug, Default, Clone)]
pub struct TextJustify {
    pub justify_horizontal: Option<TextJustifyHorizontal>,
    pub justify_vertical: Option<TextJustifyVertical>,
    pub mirror: bool,
}

#[derive(Debug, Clone)]
pub enum TextJustifyHorizontal {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum TextJustifyVertical {
    Top,
    Bottom,
}

#[derive(Debug, Default, Clone)]
pub struct Font {
    pub face: Option<String>,
    pub size: FontSize,
    pub thickness: Option<f32>,
    pub bold: bool,
    pub italic: bool,
    pub line_spacing: Option<f32>,
}

#[derive(Debug, Default, Clone)]
pub struct FontSize {
    pub width: f32,
    pub height: f32,
}

impl SyntaxItemSerializable for StrokeDefinition {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "stroke".into(),
            arguments: Vec::new(),
            children: vec![
                Some(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(self.width, PositionPreference::None))),
                self.dash.clone().map(|dash| SyntaxItem::from_single_argument("type", SyntaxArgument::Identifier(match dash {
                    StrokeType::Dash => "dash".into(),
                    StrokeType::DashDot => "dash_dot".into(),
                    StrokeType::DashDotDot => "dash_dot_dot".into(),
                    StrokeType::Dot => "dot".into(),
                    StrokeType::Default => "default".into(),
                    StrokeType::Solid => "solid".into(),
                }, PositionPreference::None))),
                self.color.as_ref().and_then(|c| Some(c.serialize())),
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        StrokeDefinition {
            width: syntax.get_named_child("width").unwrap().arguments.first().unwrap().get_number(),
            dash: syntax.get_named_child("type").map(|t| match t.arguments.first().unwrap().get_string().as_str() {
                "dash" => StrokeType::Dash,
                "dash_dot" => StrokeType::DashDot,
                "dash_dot_dot" => StrokeType::DashDotDot,
                "dot" => StrokeType::Dot,
                "default" => StrokeType::Default,
                "solid" => StrokeType::Solid,
                _ => panic!("Invalid dash type argument for StrokeDefinition"),
            }),
            color: syntax.get_named_child("color").and_then(|c| Some(Color::deserialize(c))),
        }
    }
}

impl SyntaxItemSerializable for Position {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "at".into(),
            children: Vec::new(),
            arguments: vec![
                Some(SyntaxArgument::Number(self.x, PositionPreference::None)),
                Some(SyntaxArgument::Number(self.y, PositionPreference::None)),
                self.angle.and_then(|a| Some(SyntaxArgument::Number(a, PositionPreference::None))),
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let x = syntax.arguments.get(0).unwrap().get_number();
        let y = syntax.arguments.get(1).unwrap().get_number();
        let rotation = syntax.arguments.get(2).and_then(|r| Some(r.get_number()));

        Self { x, y, angle: rotation }
    }
}

impl SyntaxItemSerializable for Id {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "id".into(),
            children: vec![],
            arguments: vec![SyntaxArgument::Number(self.id as f32, PositionPreference::None)],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self { id: syntax.arguments.get(0).unwrap().get_number() as u32 }
    }
}

impl SyntaxItemSerializable for TextEffect {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "effects".into(),
            arguments: match self.hide {
                true => vec![SyntaxArgument::Identifier("hide".into(), PositionPreference::End)],
                false => vec![],
            },
            children: vec![
                Some(self.font.serialize()),
                if self.justify.justify_vertical.is_some() || self.justify.justify_horizontal.is_some() || self.justify.mirror {
                    Some(self.justify.serialize())
                } else {
                    None
                },
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let hide = if let Some(arg) = syntax.arguments.first() {
            arg.get_string() == "hide"
        } else {
            false
        };

        let mut font = Font::default();
        let mut justify = TextJustify::default();

        for child in &syntax.children {
            match child.name.as_ref() {
                "font" => font = Font::deserialize(&child),
                "justify" => justify = TextJustify::deserialize(&child),
                _ => panic!("Invalid child element for TextEffect"),
            }
        }

        Self { hide, justify, font }
    }
}

impl SyntaxItemSerializable for TextJustify {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "justify".into(),
            children: Vec::new(),
            arguments: vec![
                self.justify_vertical.as_ref().and_then(|j| Some(SyntaxArgument::Identifier(match j {
                    TextJustifyVertical::Top => "top".into(),
                    TextJustifyVertical::Bottom => "bottom".into(),
                }, PositionPreference::None))),
                self.justify_horizontal.as_ref().and_then(|j| Some(SyntaxArgument::Identifier(match j {
                    TextJustifyHorizontal::Left => "left".into(),
                    TextJustifyHorizontal::Right => "right".into(),
                }, PositionPreference::None))),
                match self.mirror {
                    true => Some(SyntaxArgument::Identifier("mirror".into(), PositionPreference::None)),
                    false => None,
                }
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut horizontal_justify = None;
        let mut vertical_justify = None;
        let mut mirror = false;

        for argument in &syntax.arguments {
            match argument.get_string().as_str() {
                "left" => horizontal_justify = Some(TextJustifyHorizontal::Left),
                "right" => horizontal_justify = Some(TextJustifyHorizontal::Right),
                "top" => vertical_justify = Some(TextJustifyVertical::Top),
                "bottom" => vertical_justify = Some(TextJustifyVertical::Bottom),
                "mirror" => mirror = true,
                _ => panic!("Invalid argument for TextJustify"),
            }
        }

        TextJustify {
            justify_horizontal: horizontal_justify,
            justify_vertical: vertical_justify,
            mirror,
        }
    }
}

impl SyntaxItemSerializable for Font {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "font".into(),
            arguments: vec![
                if self.italic { Some(SyntaxArgument::Identifier("italic".into(), PositionPreference::None)) } else { None },
                if self.bold { Some(SyntaxArgument::Identifier("bold".into(), PositionPreference::None)) } else { None },
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
            children: vec![
                Some(self.size.serialize()),
                self.face.as_ref().and_then(|f| Some(SyntaxItem::from_single_argument("face", SyntaxArgument::QuotedString(f.clone(), PositionPreference::None)))),
                self.thickness.as_ref().and_then(|f| Some(SyntaxItem::from_single_argument("thickness", SyntaxArgument::Number(*f, PositionPreference::None)))),
                self.line_spacing.as_ref().and_then(|f| Some(SyntaxItem::from_single_argument("line_spacing", SyntaxArgument::Number(*f, PositionPreference::None)))),
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut font = Font::default();

        for arg in &syntax.arguments {
            match arg.get_string().as_str() {
                "bold" => font.bold = true,
                "italic" => font.italic = true,
                _ => panic!("Invalid argument for Font"),
            }
        }

        for child in &syntax.children {
            match child.name.as_ref() {
                "face" => font.face = Some(child.arguments.first().unwrap().get_string()),
                "thickness" => font.thickness = Some(child.arguments.first().unwrap().get_number()),
                "line_spacing" => font.line_spacing = Some(child.arguments.first().unwrap().get_number()),
                "size" => font.size = FontSize::deserialize(&child),
                "bold" => font.bold = child.arguments.first().unwrap().get_string() == "yes",
                _ => panic!("Invalid child element '{}' for Font", child.name),
            }
        }

        font
    }
}

impl SyntaxItemSerializable for FontSize {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "size".into(),
            children: Vec::new(),
            arguments: vec![
                SyntaxArgument::Number(self.width, PositionPreference::None),
                SyntaxArgument::Number(self.height, PositionPreference::None),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let width = syntax.arguments.get(0).unwrap().get_number();
        let height = syntax.arguments.get(1).unwrap().get_number();

        Self { width, height }
    }
}
