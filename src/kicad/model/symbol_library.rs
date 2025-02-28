use crate::easyeda::symbol::Object;
use crate::kicad::model::common::{Id, Position, StrokeDefinition, TextEffect, TextPosition};
use crate::kicad::syntax::{PositionPreference, SyntaxArgument, SyntaxItem, SyntaxItemSerializable, TopLevelSerializable};

#[derive(Debug)]
pub struct SymbolLib {
    pub version: usize,
    pub generator: String,
    pub generator_version: Option<String>,
    pub symbols: Vec<Symbol>,
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Symbol {
    pub symbol_id: String,
    pub extends_id: Option<String>,
    pub pin_numbers_hidden: bool,
    pub pin_names_hidden: bool,
    pub pin_names_offset: Option<f32>,
    pub in_bom: Option<bool>,
    pub on_board: Option<bool>,
    pub exclude_from_sim: Option<bool>,
    pub properties: Vec<Property>,

    pub arcs: Vec<SymbolArc>,
    pub beziers: Vec<SymbolArc>,
    pub circles: Vec<SymbolCircle>,
    pub rectangles: Vec<SymbolRectangle>,
    pub lines: Vec<SymbolLine>,
    pub curves: Vec<SymbolCurve>,
    pub texts: Vec<SymbolText>,

    pub pins: Vec<SymbolPin>,
    pub units: Vec<Symbol>,
    pub objects: Vec<Object>,
    pub unit_name: Option<String>,
}

#[derive(Debug)]
pub struct SymbolArc {
    pub start: Position,
    pub mid: Position,
    pub end: Position,
    pub stroke: StrokeDefinition,
    pub fill: FillDefinition,
}

#[derive(Debug)]
pub struct SymbolCircle {
    pub center: Position,
    pub radius: f32,
    pub stroke: StrokeDefinition,
    pub fill: FillDefinition,
}

#[derive(Debug)]
pub struct SymbolRectangle {
    pub start: Position,
    pub end: Position,
    pub stroke: StrokeDefinition,
    pub fill: FillDefinition,
}

#[derive(Debug)]
pub struct SymbolLine {
    pub points: Vec<Position>,
    pub stroke: StrokeDefinition,
    pub fill: Option<FillDefinition>,
}

#[derive(Debug)]
pub struct SymbolCurve {
    pub points: Vec<Position>,
    pub stroke: StrokeDefinition,
    pub fill: Option<FillDefinition>,
}

#[derive(Debug)]
pub struct SymbolText {
    pub text: String,
    pub position: TextPosition,
    pub effects: TextEffect,
}

#[derive(Debug)]
pub struct Property {
    pub id: Option<Id>,
    pub key: String,
    pub value: String,
    pub hide: bool,
    pub position: Position,
    pub text_effects: TextEffect,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_hex(hex_str: &str) -> Color {
        // Remove leading # if present
        let hex_str = hex_str.trim_start_matches('#');

        // Parse based on string length
        match hex_str.len() {
            3 => { // RGB format
                let r = u8::from_str_radix(&hex_str[0..1].repeat(2), 16).unwrap();
                let g = u8::from_str_radix(&hex_str[1..2].repeat(2), 16).unwrap();
                let b = u8::from_str_radix(&hex_str[2..3].repeat(2), 16).unwrap();
                Color { r, g, b, a: 255 }
            }
            4 => { // RGBA format
                let r = u8::from_str_radix(&hex_str[0..1].repeat(2), 16).unwrap();
                let g = u8::from_str_radix(&hex_str[1..2].repeat(2), 16).unwrap();
                let b = u8::from_str_radix(&hex_str[2..3].repeat(2), 16).unwrap();
                let a = u8::from_str_radix(&hex_str[3..4].repeat(2), 16).unwrap();
                Color { r, g, b, a }
            }
            6 => { // RRGGBB format
                let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap();
                let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap();
                let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap();
                Color { r, g, b, a: 255 }
            }
            8 => { // RRGGBBAA format
                let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap();
                let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap();
                let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap();
                let a = u8::from_str_radix(&hex_str[6..8], 16).unwrap();
                Color { r, g, b, a }
            }
            _ => panic!("Invalid hex color format")
        }
    }
}

#[derive(Debug, Clone)]
pub enum StrokeType {
    Dash,
    DashDot,
    DashDotDot,
    Dot,
    Default,
    Solid,
}

#[derive(Debug, Clone)]
pub enum FillType {
    None,
    Outline,
    Background,
}

#[derive(Debug, Clone)]
pub struct FillDefinition {
    pub fill_type: FillType,
}

#[derive(Debug)]
pub enum PinElectricalType {
    Input,
    Output,
    Bidirectional,
    TriState,
    Passive,
    Free,
    Unspecified,
    PowerIn,
    PowerOut,
    OpenCollector,
    OpenEmitter,
    NoConnect,
}

#[derive(Debug)]
pub enum PinGraphicStyle {
    Line,
    Inverted,
    Clock,
    InvertedClock,
    InputLow,
    ClockLow,
    OutputLow,
    EdgeClockHigh,
    NonLogic,
}

#[derive(Debug)]
pub struct SymbolPin {
    pub electrical_type: PinElectricalType,
    pub graphic_style: PinGraphicStyle,
    pub position: Position,
    pub length: f32,
    pub name: Option<String>,
    pub name_effects: TextEffect,
    pub number: Option<String>,
    pub number_effects: TextEffect,
}

impl SyntaxItemSerializable for SymbolLib {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("version", SyntaxArgument::Identifier(self.version.to_string(), PositionPreference::None)),
            SyntaxItem::from_single_argument("generator", SyntaxArgument::Identifier(self.generator.clone(), PositionPreference::None)),
        ];
        if let Some(generator_version) = &self.generator_version {
            children.push(SyntaxItem::from_single_argument("generator_version", SyntaxArgument::QuotedString(generator_version.clone(), PositionPreference::None)));
        }
        children.extend(self.symbols.iter().map(|symbol| symbol.serialize()).collect::<Vec<_>>());
        SyntaxItem {
            name: "kicad_symbol_lib".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut lib = Self {
            version: 0,
            generator: "".into(),
            generator_version: None,
            symbols: Vec::new(),
        };

        for child in syntax.children.iter() {
            match child.name.as_str() {
                "symbol" => lib.symbols.push(Symbol::deserialize(&child)),
                "version" => lib.version = child.arguments.first().unwrap().get_number() as usize,
                "generator" => lib.generator = child.arguments.first().unwrap().get_string(),
                "generator_version" => lib.generator_version = Some(child.arguments.first().unwrap().get_string()),
                _ => panic!("Unsupported child item type in SymbolLib"),
            }
        }

        lib
    }
}

impl SyntaxItemSerializable for Symbol {
    fn serialize(&self) -> SyntaxItem {
        let mut children = Vec::new();
        if self.pin_names_hidden || self.pin_names_offset.is_some() {
            children.push(SyntaxItem {
                name: "pin_names".into(),
                arguments: if self.pin_names_hidden {
                    vec![SyntaxArgument::Identifier("hide".into(), PositionPreference::End)]
                } else {
                    vec![]
                },
                children: if let Some(pin_names_offset) = self.pin_names_offset {
                    vec![SyntaxItem::from_single_argument("offset".into(), SyntaxArgument::Number(pin_names_offset, PositionPreference::None))]
                } else {
                    vec![]
                },
            })
        }
        if self.pin_numbers_hidden {
            children.push(SyntaxItem::from_single_argument("pin_numbers", SyntaxArgument::Identifier("hidden".into(), PositionPreference::None)))
        }
        if let Some(in_bom) = self.in_bom {
            children.push(SyntaxItem::from_single_argument("in_bom", SyntaxArgument::Identifier(if in_bom { "yes".into() } else { "no".into() }, PositionPreference::None)));
        }
        if let Some(on_board) = self.on_board {
            children.push(SyntaxItem::from_single_argument("on_board", SyntaxArgument::Identifier(if on_board { "yes".into() } else { "no".into() }, PositionPreference::None)));
        }

        if let Some(extends_id) = &self.extends_id {
            children.push(SyntaxItem::from_single_argument("extends", SyntaxArgument::QuotedString(extends_id.clone(), PositionPreference::None)));
        }
        if let Some(unit_name) = &self.unit_name {
            children.push(SyntaxItem::from_single_argument("unit_name", SyntaxArgument::QuotedString(unit_name.clone(), PositionPreference::None)));
        }
        if let Some(exclude_from_sim) = self.exclude_from_sim {
            children.push(SyntaxItem::from_single_argument("exclude_from_sim", SyntaxArgument::Identifier(if exclude_from_sim { "yes".into() } else { "no".into() }, PositionPreference::None)));
        }

        children.extend(self.properties.iter().map(|property| property.serialize()).collect::<Vec<_>>());
        children.extend(self.arcs.iter().map(|arc| arc.serialize()).collect::<Vec<_>>());
        children.extend(self.circles.iter().map(|circle| circle.serialize()).collect::<Vec<_>>());
        children.extend(self.curves.iter().map(|curve| curve.serialize()).collect::<Vec<_>>());
        children.extend(self.lines.iter().map(|line| line.serialize()).collect::<Vec<_>>());
        children.extend(self.rectangles.iter().map(|rectangle| rectangle.serialize()).collect::<Vec<_>>());
        children.extend(self.texts.iter().map(|text| text.serialize()).collect::<Vec<_>>());
        children.extend(self.pins.iter().map(|pin| pin.serialize()).collect::<Vec<_>>());

        for child_symbol in &self.units {
            children.push(child_symbol.serialize());
        }

        SyntaxItem {
            name: "symbol".into(),
            arguments: vec![SyntaxArgument::QuotedString(self.symbol_id.clone(), PositionPreference::None)],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let name = syntax.arguments.first().unwrap().get_string();

        let mut symbol = Self {
            symbol_id: name,
            in_bom: None,
            on_board: None,
            exclude_from_sim: None,
            properties: Vec::new(),
            pins: Vec::new(),
            arcs: Vec::new(),
            beziers: Vec::new(),
            circles: Vec::new(),
            rectangles: Vec::new(),
            lines: Vec::new(),
            curves: Vec::new(),
            texts: Vec::new(),
            objects: Vec::new(),
            units: Vec::new(),
            extends_id: None,
            unit_name: None,
            pin_numbers_hidden: false,
            pin_names_hidden: false,
            pin_names_offset: None,
        };

        for child in syntax.children.iter() {
            match child.name.as_str() {
                "property" => symbol.properties.push(Property::deserialize(&child)),
                "pin" => symbol.pins.push(SymbolPin::deserialize(&child)),
                "arc" => symbol.arcs.push(SymbolArc::deserialize(&child)),
                "circle" => symbol.circles.push(SymbolCircle::deserialize(&child)),
                "bezier" => symbol.curves.push(SymbolCurve::deserialize(&child)),
                "polyline" => symbol.lines.push(SymbolLine::deserialize(&child)),
                "rectangle" => symbol.rectangles.push(SymbolRectangle::deserialize(&child)),
                "text" => symbol.texts.push(SymbolText::deserialize(&child)),
                "in_bom" => symbol.in_bom = Some(child.arguments.first().unwrap().get_string() == "yes"),
                "on_board" => symbol.on_board = Some(child.arguments.first().unwrap().get_string() == "yes"),
                "exclude_from_sim" => symbol.exclude_from_sim = Some(child.arguments.first().unwrap().get_string() == "yes"),
                "extends" => symbol.extends_id = Some(child.arguments.first().unwrap().get_string()),
                "unit_name" => symbol.unit_name = Some(child.arguments.first().unwrap().get_string()),
                "pin_numbers" => symbol.pin_numbers_hidden = child.arguments.first().unwrap().get_string() == "hidden",
                "pin_names" => {
                    symbol.pin_names_hidden = child.has_argument(SyntaxArgument::Identifier("hide".into(), PositionPreference::None));
                    symbol.pin_names_offset = child.get_named_child("offset".into())
                        .and_then(|c| Some(c.arguments.first().unwrap().get_number()))
                }
                "symbol" => symbol.units.push(Symbol::deserialize(&child)),
                "embedded_fonts" => {},
                _ => panic!("Unsupported child item type in Symbol: {}", child.name)
            }
        }

        symbol
    }
}

impl SyntaxItemSerializable for Property {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![self.position.serialize(), self.text_effects.serialize()];

        if let Some(id) = &self.id {
            children.insert(0, id.serialize());
        }

        if self.hide {
            children.push(SyntaxItem::from_single_argument("hide", SyntaxArgument::Identifier("yes".into(), PositionPreference::None)));
        }

        SyntaxItem {
            name: "property".into(),
            arguments: vec![
                SyntaxArgument::QuotedString(self.key.clone(), PositionPreference::None),
                SyntaxArgument::QuotedString(self.value.clone(), PositionPreference::None),
            ],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let key = syntax.arguments.get(0).unwrap().get_string();
        let value = syntax.arguments.get(1).unwrap().get_string();
        let effects = syntax.get_named_child("effects").unwrap();
        let id = syntax.get_named_child("id");

        Self {
            id: id.map(|s| Id::deserialize(s)),
            key,
            value,
            hide: syntax.get_named_child("hide").is_some_and(|c| c.arguments.first().unwrap().get_string() == "yes"),
            text_effects: TextEffect::deserialize(&effects),
            position: Position::deserialize(syntax.get_named_child("at").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for SymbolPin {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "pin".into(),
            arguments: vec![
                SyntaxArgument::Identifier(match self.electrical_type {
                    PinElectricalType::Input => "input".into(),
                    PinElectricalType::Output => "output".into(),
                    PinElectricalType::Bidirectional => "bidirectional".into(),
                    PinElectricalType::TriState => "tri_state".into(),
                    PinElectricalType::Passive => "passive".into(),
                    PinElectricalType::Free => "free".into(),
                    PinElectricalType::Unspecified => "unspecified".into(),
                    PinElectricalType::PowerIn => "power_in".into(),
                    PinElectricalType::PowerOut => "power_out".into(),
                    PinElectricalType::OpenCollector => "open_collector".into(),
                    PinElectricalType::OpenEmitter => "open_emitter".into(),
                    PinElectricalType::NoConnect => "no_connect".into(),
                }, PositionPreference::None),
                SyntaxArgument::Identifier(match self.graphic_style {
                    PinGraphicStyle::Line => "line".into(),
                    PinGraphicStyle::Inverted => "inverted".into(),
                    PinGraphicStyle::Clock => "clock".into(),
                    PinGraphicStyle::InvertedClock => "inverted_clock".into(),
                    PinGraphicStyle::InputLow => "input_low".into(),
                    PinGraphicStyle::ClockLow => "output_low".into(),
                    PinGraphicStyle::OutputLow => "clock_low".into(),
                    PinGraphicStyle::EdgeClockHigh => "edge_clock_high".into(),
                    PinGraphicStyle::NonLogic => "non_logic".into(),
                }, PositionPreference::None)
            ],
            children: vec![
                Some(self.position.serialize()),
                Some(SyntaxItem::from_single_argument("length", SyntaxArgument::Number(self.length, PositionPreference::None))),
                self.name.as_ref().and_then(|n| Some(SyntaxItem {
                    name: "name".into(),
                    arguments: vec![SyntaxArgument::QuotedString(n.to_string(), PositionPreference::None), ],
                    children: vec![self.name_effects.serialize()],
                })),
                self.number.as_ref().and_then(|n| Some(SyntaxItem {
                    name: "number".into(),
                    arguments: vec![SyntaxArgument::QuotedString(n.to_string(), PositionPreference::None), ],
                    children: vec![self.number_effects.serialize()],
                })),
            ].iter().filter(|&o| o.is_some()).map(|o| o.as_ref().unwrap().clone()).collect(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut pin = SymbolPin {
            electrical_type: PinElectricalType::Unspecified,
            graphic_style: PinGraphicStyle::Line,
            number: None,
            name: None,
            length: 0.0,
            position: Position { x: 0.0, y: 0.0, angle: None },
            name_effects: TextEffect::default(),
            number_effects: TextEffect::default(),
        };

        pin.electrical_type = match syntax.arguments.get(0).unwrap().get_string().as_str() {
            "input" => PinElectricalType::Input,
            "output" => PinElectricalType::Output,
            "bidirectional" => PinElectricalType::Bidirectional,
            "tri_state" => PinElectricalType::TriState,
            "passive" => PinElectricalType::Passive,
            "free" => PinElectricalType::Free,
            "unspecified" => PinElectricalType::Unspecified,
            "power_in" => PinElectricalType::PowerIn,
            "power_out" => PinElectricalType::PowerOut,
            "open_collector" => PinElectricalType::OpenCollector,
            "open_emitter" => PinElectricalType::OpenEmitter,
            "no_connect" => PinElectricalType::NoConnect,
            _ => panic!("Invalid electrical type argument for SymbolPin"),
        };

        pin.graphic_style = match syntax.arguments.get(1).unwrap().get_string().as_str() {
            "line" => PinGraphicStyle::Line,
            "inverted" => PinGraphicStyle::Inverted,
            "clock" => PinGraphicStyle::Clock,
            "inverted_clock" => PinGraphicStyle::InvertedClock,
            "input_low" => PinGraphicStyle::InputLow,
            "output_low" => PinGraphicStyle::OutputLow,
            "clock_low" => PinGraphicStyle::ClockLow,
            "edge_clock_high" => PinGraphicStyle::EdgeClockHigh,
            "non_logic" => PinGraphicStyle::NonLogic,
            _ => panic!("Invalid graphic style argument for SymbolPin"),
        };

        for child in &syntax.children {
            match child.name.as_ref() {
                "at" => pin.position = Position::deserialize(&child),
                "length" => pin.length = child.arguments.first().unwrap().get_number(),
                "number" => {
                    pin.number = Some(child.arguments.first().unwrap().get_string());
                    pin.number_effects = TextEffect::deserialize(&child.children.first().unwrap());
                }
                "name" => {
                    pin.name = Some(child.arguments.first().unwrap().get_string());
                    pin.name_effects = TextEffect::deserialize(&child.children.first().unwrap());
                }
                _ => panic!("Invalid child element for SymbolPin"),
            }
        }

        pin
    }
}

impl SyntaxItemSerializable for SymbolArc {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "arc".into(),
            arguments: Vec::new(),
            children: vec![
                SyntaxItem::from_arguments("start", vec![
                    SyntaxArgument::Number(self.start.x, PositionPreference::None),
                    SyntaxArgument::Number(self.start.y, PositionPreference::None),
                ]),
                SyntaxItem::from_arguments("mid", vec![
                    SyntaxArgument::Number(self.mid.x, PositionPreference::None),
                    SyntaxArgument::Number(self.mid.y, PositionPreference::None),
                ]),
                SyntaxItem::from_arguments("end", vec![
                    SyntaxArgument::Number(self.end.x, PositionPreference::None),
                    SyntaxArgument::Number(self.end.y, PositionPreference::None),
                ]),
                self.fill.serialize(),
                self.stroke.serialize(),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            start: Position::deserialize(syntax.get_named_child("start").unwrap()),
            mid: Position::deserialize(syntax.get_named_child("mid").unwrap()),
            end: Position::deserialize(syntax.get_named_child("end").unwrap()),
            fill: FillDefinition::deserialize(syntax.get_named_child("fill").unwrap()),
            stroke: StrokeDefinition::deserialize(syntax.get_named_child("stroke").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for SymbolCircle {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "circle".into(),
            arguments: Vec::new(),
            children: vec![
                SyntaxItem::from_arguments("center", vec![
                    SyntaxArgument::Number(self.center.x, PositionPreference::None),
                    SyntaxArgument::Number(self.center.y, PositionPreference::None),
                ]),
                SyntaxItem::from_single_argument("radius", SyntaxArgument::Number(self.radius, PositionPreference::None)),
                self.fill.serialize(),
                self.stroke.serialize(),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            center: Position::deserialize(syntax.get_named_child("center").unwrap()),
            radius: syntax.get_named_child("radius").unwrap().arguments.first().unwrap().get_number(),
            fill: FillDefinition::deserialize(syntax.get_named_child("fill").unwrap()),
            stroke: StrokeDefinition::deserialize(syntax.get_named_child("stroke").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for SymbolRectangle {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "rectangle".into(),
            arguments: Vec::new(),
            children: vec![
                SyntaxItem::from_arguments("start", vec![
                    SyntaxArgument::Number(self.start.x, PositionPreference::None),
                    SyntaxArgument::Number(self.start.y, PositionPreference::None),
                ]),
                SyntaxItem::from_arguments("end", vec![
                    SyntaxArgument::Number(self.end.x, PositionPreference::None),
                    SyntaxArgument::Number(self.end.y, PositionPreference::None),
                ]),
                self.stroke.serialize(),
                self.fill.serialize(),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            start: Position::deserialize(syntax.get_named_child("start").unwrap()),
            end: Position::deserialize(syntax.get_named_child("end").unwrap()),
            fill: FillDefinition::deserialize(syntax.get_named_child("fill").unwrap()),
            stroke: StrokeDefinition::deserialize(syntax.get_named_child("stroke").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for SymbolLine {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem {
                name: "pts".into(),
                arguments: Vec::new(),
                children: self.points
                    .iter()
                    .map(|p| SyntaxItem::from_arguments("xy", vec![
                        SyntaxArgument::Number(p.x, PositionPreference::None),
                        SyntaxArgument::Number(p.y, PositionPreference::None),
                    ]))
                    .collect(),
            },
            self.stroke.serialize(),
        ];

        if let Some(fill) = &self.fill {
            children.push(fill.serialize());
        }

        SyntaxItem {
            name: "polyline".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            points: syntax.get_named_child("pts").unwrap().children
                .iter().map(|child| Position::deserialize(child)).collect(),
            fill: syntax.get_named_child("fill").map(|f| FillDefinition::deserialize(f)),
            stroke: StrokeDefinition::deserialize(syntax.get_named_child("stroke").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for SymbolCurve {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "bezier".into(),
            arguments: Vec::new(),
            children: SymbolLine {
                points: self.points.clone(),
                stroke: self.stroke.clone(),
                fill: self.fill.clone(),
            }.serialize().children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            points: syntax.get_named_child("pts").unwrap().children
                .iter().map(|child| Position::deserialize(child)).collect(),
            fill: syntax.get_named_child("fill").map(|f| FillDefinition::deserialize(f)),
            stroke: StrokeDefinition::deserialize(syntax.get_named_child("stroke").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for SymbolText {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "text".into(),
            arguments: vec![SyntaxArgument::QuotedString(self.text.clone(), PositionPreference::None)],
            children: vec![self.position.serialize(), self.effects.serialize()],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            text: syntax.arguments.first().unwrap().get_string(),
            position: TextPosition::deserialize(syntax.get_named_child("at").unwrap()),
            effects: TextEffect::deserialize(syntax.get_named_child("effects").unwrap()),
        }
    }
}

impl SyntaxItemSerializable for FillDefinition {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "fill".into(),
            children: vec![SyntaxItem {
                name: "type".into(),
                children: Vec::new(),
                arguments: vec![SyntaxArgument::Identifier(match self.fill_type {
                    FillType::None => "none".into(),
                    FillType::Outline => "outline".into(),
                    FillType::Background => "background".into(),
                }, PositionPreference::None)],
            }],
            arguments: Vec::new(),
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        FillDefinition {
            fill_type: match syntax.get_named_child("type").unwrap().arguments.first().unwrap().get_string().as_str() {
                "none" => FillType::None,
                "outline" => FillType::Outline,
                "background" => FillType::Background,
                _ => panic!("Invalid fill type argument for FillDefinition"),
            }
        }
    }
}

impl SyntaxItemSerializable for Color {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "color".into(),
            children: Vec::new(),
            arguments: vec![
                SyntaxArgument::Number(self.r as f32, PositionPreference::None),
                SyntaxArgument::Number(self.g as f32, PositionPreference::None),
                SyntaxArgument::Number(self.b as f32, PositionPreference::None),
                SyntaxArgument::Number(self.a as f32, PositionPreference::None),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            r: syntax.arguments.iter().nth(0).unwrap().get_number() as u8,
            g: syntax.arguments.iter().nth(1).unwrap().get_number() as u8,
            b: syntax.arguments.iter().nth(2).unwrap().get_number() as u8,
            a: syntax.arguments.iter().nth(3).unwrap().get_number() as u8,
        }
    }
}

impl TopLevelSerializable for SymbolLib {
    fn get_same_line_identifiers() -> Vec<String> {
        Vec::from([
            "version", "generator", "at", "font", "size", "justify", "width", "type", "in_bom",
            "on_board", "length", "extends", "unit_name", "pin_names", "offset", "start", "end",
            "thickness"
        ]).iter().map(|s| s.to_string()).collect()
    }
}
