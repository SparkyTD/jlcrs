use crate::easyeda::geometry::Point2D;
use crate::easyeda::json_reader::JsonArrayReader;
use crate::easyeda::errors::{ParserError, ParserType, SymbolConverterError};
use crate::kicad::model::common::{FontSize, Position, StrokeDefinition, TextEffect, TextJustifyHorizontal, TextJustifyVertical, TextPosition};
use crate::kicad::model::symbol_library::{Color, FillDefinition, FillType, PinElectricalType, PinGraphicStyle, StrokeType, Symbol, SymbolArc, SymbolCircle, SymbolLib, SymbolLine, SymbolPin, SymbolRectangle, SymbolText};
use itertools::Itertools;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub struct EasyEDASymbol {
    pub elements: Vec<SymbolElement>,

    pub part_number: Option<String>,
}

impl EasyEDASymbol {
    pub fn parse(symbol_data: &str) -> anyhow::Result<EasyEDASymbol> {
        let mut elements = Vec::new();

        for param in symbol_data.split_terminator(['\r', '\n']) {
            if param.len() == 0 {
                continue;
            }

            let prop = SymbolElement::parse_line(param)?;
            if prop.is_none() {
                continue;
            }
            let prop = prop.unwrap();

            elements.push(prop);
        }

        Ok(Self {
            part_number: None,
            elements,
        })
    }

    pub fn get_designator(&self) -> Option<String> {
        for element in &self.elements {
            if let SymbolElement::ATTR(attribute) = element {
                if attribute.key != "Designator" {
                    continue;
                }

                if attribute.parent_id.clone().is_none_or(|s| s.is_empty()) {
                    return attribute.value.clone();
                }
            }
        }

        None
    }
}

impl TryInto<SymbolLib> for EasyEDASymbol {
    type Error = SymbolConverterError;

    fn try_into(self) -> Result<SymbolLib, Self::Error> {
        let symbol = self.try_into()?;
        Ok(SymbolLib {
            version: 20211014,
            generator: "easyeda-to-kicad".into(),
            generator_version: None,
            symbols: vec![symbol],
        })
    }
}

impl TryInto<Symbol> for EasyEDASymbol {
    type Error = SymbolConverterError;

    fn try_into(self) -> Result<Symbol, Self::Error> {
        let default_text_effect = TextEffect::default();

        let scale_factor = 0.254;

        let mut line_styles = HashMap::new();
        let mut text_styles = HashMap::new();
        let mut current_part = Part::default();
        let mut attributes = Vec::new();

        let all_parts = self.elements.iter()
            .filter_map(|e| match e {
                SymbolElement::PART(part) => Some(part),
                _ => None,
            }).collect_vec();
        let is_complex_symbol = all_parts.len() > 1;

        let mut all_symbols = Vec::new();
        let mut current_symbol_index = usize::MAX;

        // Extract all attributes
        for element in &self.elements {
            match element {
                SymbolElement::ATTR(attribute) => {
                    if let Some(_) = &attribute.parent_id {
                        attributes.push(attribute.clone());
                    }
                }
                _ => {}
            }
        }

        // Parse and convert elements
        for element in self.elements {
            let attribute_key = match &element {
                SymbolElement::ATTR(e) => Some(e.id.clone()),
                SymbolElement::PART(e) => Some(e.id.clone()),
                SymbolElement::RECT(e) => Some(e.id.clone()),
                SymbolElement::CIRCLE(e) => Some(e.id.clone()),
                SymbolElement::POLYLINE(e) => Some(e.id.clone()),
                SymbolElement::ARC(e) => Some(e.id.clone()),
                SymbolElement::TEXT(e) => Some(e.id.clone()),
                SymbolElement::PIN(e) => Some(e.id.clone()),

                _ => None
            };
            let attributes = attributes.iter().filter(|f| f.parent_id == attribute_key).collect_vec();

            match element {
                SymbolElement::LINESTYLE(style) => {
                    line_styles.insert(style.index_name.clone(), style);
                }
                SymbolElement::FONTSTYLE(style) => {
                    text_styles.insert(style.index_name.clone(), style);
                }
                SymbolElement::PART(part) => {
                    let symbol = Symbol {
                        symbol_id: part.id.clone(),
                        in_bom: Some(true),
                        on_board: Some(true),
                        ..Default::default()
                    };
                    all_symbols.push(symbol);
                    current_symbol_index = all_symbols.len() - 1;
                    current_part = part;
                }
                SymbolElement::ATTR(attr) => {
                    match attr.parent_id {
                        None => current_part.attributes.push(attr),
                        _ => {}
                    }
                }
                SymbolElement::RECT(rectangle) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let line_style = line_styles.get(&rectangle.style_id.unwrap()).unwrap();
                    current_symbol.rectangles.push(SymbolRectangle {
                        start: Position { x: rectangle.x * scale_factor, y: rectangle.y * scale_factor, angle: None },
                        end: Position { x: rectangle.end_x * scale_factor, y: rectangle.end_y * scale_factor, angle: None },
                        stroke: StrokeDefinition {
                            width: line_style.stroke_width.unwrap_or(0.254),
                            color: line_style.stroke_color.clone().and_then(|s| Some(Color::from_hex(&s))),
                            dash: Some(StrokeType::Solid),
                        },
                        fill: FillDefinition {
                            fill_type: FillType::Background,
                        },
                    });
                }
                SymbolElement::CIRCLE(circle) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let line_style = line_styles.get(&circle.style_id.unwrap()).unwrap();
                    current_symbol.circles.push(SymbolCircle {
                        center: Position { x: circle.cx * scale_factor, y: circle.cy * scale_factor, angle: None },
                        radius: circle.radius * scale_factor,
                        stroke: StrokeDefinition {
                            width: line_style.stroke_width.unwrap_or(0.254),
                            color: line_style.stroke_color.clone().and_then(|s| Some(Color::from_hex(&s))),
                            dash: Some(StrokeType::Solid),
                        },
                        fill: FillDefinition {
                            fill_type: FillType::Outline,
                        },
                    });
                }
                SymbolElement::ELLIPSE(ellipse) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let line_style = line_styles.get(&ellipse.style_id.unwrap()).unwrap();
                    if ellipse.radius_x == ellipse.radius_y {
                        current_symbol.circles.push(SymbolCircle {
                            center: Position { x: ellipse.cx * scale_factor, y: ellipse.cy * scale_factor, angle: None },
                            radius: ellipse.radius_x * scale_factor,
                            stroke: StrokeDefinition {
                                width: line_style.stroke_width.unwrap_or(0.254),
                                color: line_style.stroke_color.clone().and_then(|s| Some(Color::from_hex(&s))),
                                dash: Some(StrokeType::Solid),
                            },
                            fill: FillDefinition {
                                fill_type: FillType::Outline,
                            },
                        });
                    } else {
                        return Err(SymbolConverterError::UnsupportedElement("Ellipse".into()));
                    }
                }
                SymbolElement::POLYLINE(line) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let line_style = line_styles.get(&line.style_id.unwrap()).unwrap();
                    current_symbol.lines.push(SymbolLine {
                        points: line.points.iter().map(|p| Position { x: p.0 * scale_factor, y: p.1 * scale_factor, angle: None }).collect(),
                        stroke: StrokeDefinition {
                            width: line_style.stroke_width.unwrap_or(0.254),
                            color: line_style.stroke_color.clone().and_then(|s| Some(Color::from_hex(&s))),
                            dash: Some(StrokeType::Solid),
                        },
                        fill: Some(FillDefinition {
                            fill_type: FillType::None,
                        }),
                    });
                }
                SymbolElement::ARC(arc) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let line_style = line_styles.get(&arc.style_id.unwrap()).unwrap();
                    current_symbol.arcs.push(SymbolArc {
                        start: Position { x: arc.x1 * scale_factor, y: arc.y1 * scale_factor, angle: None },
                        mid: Position { x: arc.x2 * scale_factor, y: arc.y2 * scale_factor, angle: None },
                        end: Position { x: arc.x3 * scale_factor, y: arc.y3 * scale_factor, angle: None },
                        stroke: StrokeDefinition {
                            width: line_style.stroke_width.unwrap_or(0.254),
                            color: line_style.stroke_color.clone().and_then(|s| Some(Color::from_hex(&s))),
                            dash: Some(StrokeType::Solid),
                        },
                        fill: FillDefinition {
                            fill_type: FillType::None,
                        },
                    });
                }
                SymbolElement::BEZIER(_bezier) => {
                    // todo implement bezier
                    return Err(SymbolConverterError::UnsupportedElement("Bezier".into()));
                }
                SymbolElement::TEXT(text) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let mut text_style = default_text_effect.clone();
                    if let Some(style) = text.style_id.and_then(|id| text_styles.get(&id)) {
                        text_style.font.bold = style.is_bold.is_some_and(|b| b);
                        text_style.font.italic = style.is_italic.is_some_and(|b| b);

                        if let Some(size) = style.font_size {
                            text_style.font.size = FontSize {
                                width: size * scale_factor * 0.5,
                                height: size * scale_factor * 0.5,
                            }
                        }

                        text_style.justify.justify_horizontal = style.h_align.map(|a| match a {
                            0 | 1 => TextJustifyHorizontal::Left,
                            _ => TextJustifyHorizontal::Right,
                        });

                        text_style.justify.justify_vertical = style.v_align.map(|a| match a {
                            0 | 1 => TextJustifyVertical::Top,
                            _ => TextJustifyVertical::Bottom,
                        });
                    }

                    current_symbol.texts.push(SymbolText {
                        text: text.text,
                        position: TextPosition { x: text.x * scale_factor, y: text.y * scale_factor, angle: Some(text.rotation) },
                        effects: text_style,
                    });
                }
                SymbolElement::PIN(pin) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    let number_attr = attributes.iter().filter(|a| a.key == "NUMBER").last().unwrap();
                    let name_attr = attributes.iter().filter(|a| a.key == "NAME").last().unwrap();

                    let number = number_attr.value.clone().unwrap();
                    let mut name = name_attr.value.clone().unwrap();
                    if number == name {
                        name = "~".into();
                    }

                    current_symbol.pins.push(SymbolPin {
                        position: Position { x: pin.x * scale_factor, y: pin.y * scale_factor, angle: Some(pin.rotation) },
                        length: pin.length * scale_factor,
                        number: Some(number),
                        name: Some(name),
                        name_effects: default_text_effect.clone(),
                        number_effects: default_text_effect.clone(),
                        graphic_style: match pin.pin_shape {
                            PinShape::None => PinGraphicStyle::Line,
                            PinShape::Clock => PinGraphicStyle::Clock,
                            PinShape::Inverted => PinGraphicStyle::Inverted,
                            PinShape::InvertedClock => PinGraphicStyle::InvertedClock,
                        },
                        electrical_type: PinElectricalType::Unspecified,
                    });
                }
                SymbolElement::OBJ(obj) => {
                    let current_symbol = all_symbols.get_mut(current_symbol_index).unwrap();
                    current_symbol.objects.push(obj);
                }

                SymbolElement::DOCTYPE(_) | SymbolElement::HEAD(_) => {}
            }
        }

        let mut root_symbol = Symbol {
            in_bom: Some(true),
            on_board: Some(true),
            ..Default::default()
        };

        if is_complex_symbol {
            let unit_symbol_names = all_symbols.iter()
                .map(|s| s.symbol_id.clone())
                .collect_vec();
            let name_parts = unit_symbol_names.iter()
                .map(|s| match s.rfind('.') {
                    Some(index) => Some((s[..index].to_string(), s[index + 1..].parse::<usize>())),
                    None => None,
                }).collect_vec();

            // Check overall sequence formatting
            if name_parts.iter().any(|p| p.clone().is_none_or(|(_, id)| id.is_err())) {
                return Err(SymbolConverterError::IncorrectUnitFormat(format!("{:?}", name_parts)));
            }

            let name_parts = name_parts.into_iter()
                .map(|p| (p.clone().unwrap().0, p.unwrap().1.unwrap()))
                .collect_vec();

            // Check number sequence suffixes
            let sequence_okay = name_parts.iter()
                .map(|p| p.1).zip_eq(1..name_parts.len() + 1)
                .all(|(a, b)| a == b);
            if !sequence_okay {
                return Err(SymbolConverterError::IncorrectUnitNumIdentifier(format!("{:?}", name_parts)));
            }

            // Check base part matches
            if !name_parts.windows(2).all(|w| w[0].0 == w[1].0) {
                return Err(SymbolConverterError::IncorrectUnitName(format!("{:?}", name_parts)));
            }

            root_symbol.symbol_id = name_parts.first().unwrap().0.clone();
            let mut unit_index = 1;
            for mut unit_symbol in all_symbols {
                unit_symbol.symbol_id = format!("{}_{}_1", root_symbol.symbol_id, unit_index);
                unit_symbol.in_bom = None;
                unit_symbol.on_board = None;
                root_symbol.units.push(unit_symbol);
                unit_index += 1;
            }
        } else {
            let mut symbol = all_symbols.pop().unwrap();
            symbol.in_bom = Some(true);
            symbol.on_board = Some(true);
            root_symbol = symbol;
        }

        // todo add basic properties to root

        Ok(root_symbol)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocType {
    pub kind: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    pub symbol_type: u32, // must be 2
    pub origin_x: f32,
    pub origin_y: f32,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineStyle {
    pub index_name: String,
    pub stroke_color: Option<String>,
    pub stroke_style: Option<u8>, // 0 | 1 | 2 | 3
    pub fill_color: Option<String>,
    pub stroke_width: Option<f32>,
    pub fill_style: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontStyle {
    pub index_name: String,
    pub fill_color: Option<String>,
    pub color: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub is_italic: Option<bool>,
    pub is_bold: Option<bool>,
    pub is_underline: Option<bool>,
    pub is_strikethrough: Option<bool>,
    pub v_align: Option<u8>,
    pub h_align: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Part {
    pub id: String,
    pub bbox_x: f32,
    pub bbox_y: f32,
    pub bbox_end_x: f32,
    pub bbox_end_y: f32,

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Attribute {
    pub id: String,
    pub parent_id: Option<String>,
    pub key: String,
    pub value: Option<String>,
    pub key_visible: Option<bool>,
    pub value_visible: Option<bool>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub rotation: Option<f32>,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rectangle {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub rx: f32,
    pub ry: f32,
    pub rotation: f32,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Circle {
    pub id: String,
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ellipse {
    pub id: String,
    pub cx: f32,
    pub cy: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub unknown: Value,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolyLine {
    pub id: String,
    pub points: Vec<(f32, f32)>,
    pub is_closed: bool,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arc {
    pub id: String,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub x3: f32,
    pub y3: f32,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bezier {
    pub id: String,
    pub control_points: Vec<Point2D>,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize, FromPrimitive)]
pub enum PinShape {
    None = 0,
    Clock = 1,
    Inverted = 2,
    InvertedClock = 3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pin {
    pub id: String,
    pub display: bool,
    pub electric: Option<bool>,
    pub x: f32,
    pub y: f32,
    pub length: f32,
    pub rotation: f32,
    pub pin_color: Option<String>,
    pub pin_shape: PinShape,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Text {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub text: String,
    pub style_id: Option<String>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub id: String,
    pub file_name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub is_mirrored: bool,
    pub data_url: String,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SymbolElement {
    DOCTYPE(DocType),
    HEAD(Head),
    LINESTYLE(LineStyle),
    FONTSTYLE(FontStyle),
    PART(Part),
    ATTR(Attribute),
    RECT(Rectangle),
    CIRCLE(Circle),
    ELLIPSE(Ellipse),
    POLYLINE(PolyLine),
    ARC(Arc),
    BEZIER(Bezier),
    PIN(Pin),
    TEXT(Text),
    OBJ(Object),
}

impl SymbolElement {
    pub fn parse_line(line: &str) -> Result<Option<Self>, ParserError> {
        let array: Vec<Value> = serde_json::from_str(line)?;
        let mut reader = JsonArrayReader::new(array);

        if !reader.can_read() {
            return Ok(None);
        }

        let property_type = reader.read_string()
            .ok_or_else(|| ParserError::InvalidPropertyType(ParserType::Symbol, "Invalid type".to_string()))?;

        match property_type.as_str() {
            "DOCTYPE" => {
                if reader.remaining() != 2 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::DOCTYPE(DocType {
                    kind: reader.read_string().unwrap(),
                    version: reader.read_string().unwrap(),
                })))
            }
            "HEAD" => {
                if reader.remaining() != 1 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                let parameters = reader.read_value().unwrap();

                Ok(Some(SymbolElement::HEAD(Head {
                    symbol_type: parameters["symbolType"].to_string().parse::<u32>().map_err(|e| ParserError::FormatError(ParserType::Symbol, e.to_string()))?,
                    version: parameters["version"].as_str().unwrap().to_string(),
                    origin_x: parameters["originX"].to_string().parse::<f32>().map_err(|e| ParserError::FormatError(ParserType::Symbol, e.to_string()))?,
                    origin_y: parameters["originY"].to_string().parse::<f32>().map_err(|e| ParserError::FormatError(ParserType::Symbol, e.to_string()))?,
                })))
            }
            "LINESTYLE" => {
                if reader.remaining() != 6 && reader.remaining() != 5 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::LINESTYLE(LineStyle {
                    index_name: reader.read_string().unwrap(),
                    stroke_color: reader.read_string(),
                    stroke_style: reader.read_u8(),
                    fill_color: reader.read_string(),
                    stroke_width: reader.read_f32(),
                    fill_style: if reader.can_read() { reader.read_string() } else { None },
                })))
            }
            "FONTSTYLE" => {
                if reader.remaining() != 11 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::FONTSTYLE(FontStyle {
                    index_name: reader.read_string().unwrap(),
                    fill_color: reader.read_string(),
                    color: reader.read_string(),
                    font_family: reader.read_string(),
                    font_size: reader.read_f32(),
                    is_italic: reader.read_bool(),
                    is_bold: reader.read_bool(),
                    is_underline: reader.read_bool(),
                    is_strikethrough: reader.read_bool(),
                    v_align: reader.read_u8(),
                    h_align: reader.read_u8(),
                })))
            }
            "PART" => {
                if reader.remaining() != 2 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                let id = reader.read_string().unwrap();
                let bbox = reader.read_value().unwrap();
                let bbox: Vec<Value> = bbox["BBOX"].as_array().unwrap().to_vec();

                Ok(Some(SymbolElement::PART(Part {
                    id,
                    bbox_x: bbox[0].as_f64().map(|f| f as f32).unwrap(),
                    bbox_y: bbox[1].as_f64().map(|f| f as f32).unwrap(),
                    bbox_end_x: bbox[2].as_f64().map(|f| f as f32).unwrap(),
                    bbox_end_y: bbox[3].as_f64().map(|f| f as f32).unwrap(),

                    attributes: Vec::new(),
                })))
            }
            "ATTR" => {
                if reader.remaining() != 11 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::ATTR(Attribute {
                    id: reader.read_string().unwrap(),
                    parent_id: reader.read_string().and_then(|s| if s.len() == 0 { None } else { Some(s.to_string()) }),
                    key: reader.read_string().unwrap(),
                    value: reader.read_string(),
                    key_visible: reader.read_bool(),
                    value_visible: reader.read_bool(),
                    x: reader.read_f32(),
                    y: reader.read_f32(),
                    rotation: reader.read_f32(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "RECT" => {
                if reader.remaining() != 10 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::RECT(Rectangle {
                    id: reader.read_string().unwrap(),
                    x: reader.read_f32().unwrap(),
                    y: reader.read_f32().unwrap(),
                    end_x: reader.read_f32().unwrap(),
                    end_y: reader.read_f32().unwrap(),
                    rx: reader.read_f32().unwrap(),
                    ry: reader.read_f32().unwrap(),
                    rotation: reader.read_f32().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "CIRCLE" => {
                if reader.remaining() != 6 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::CIRCLE(Circle {
                    id: reader.read_string().unwrap(),
                    cx: reader.read_f32().unwrap(),
                    cy: reader.read_f32().unwrap(),
                    radius: reader.read_f32().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "ELLIPSE" => {
                if reader.remaining() != 8 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::ELLIPSE(Ellipse {
                    id: reader.read_string().unwrap(),
                    cx: reader.read_f32().unwrap(),
                    cy: reader.read_f32().unwrap(),
                    radius_x: reader.read_f32().unwrap(),
                    radius_y: reader.read_f32().unwrap(),
                    unknown: reader.read_value().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "POLY" => {
                if reader.remaining() != 5 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                let id = reader.read_string().unwrap();
                let point_array = reader.read_value().unwrap();
                let point_array = point_array.as_array().unwrap();

                Ok(Some(SymbolElement::POLYLINE(PolyLine {
                    id,
                    points: point_array.chunks(2)
                        .map(|a| (a[0].as_f64().unwrap() as f32, a[1].as_f64().unwrap() as f32))
                        .collect(),
                    is_closed: reader.read_bool().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "ARC" => {
                if reader.remaining() != 9 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::ARC(Arc {
                    id: reader.read_string().unwrap(),
                    x1: reader.read_f32().unwrap(),
                    y1: reader.read_f32().unwrap(),
                    x2: reader.read_f32().unwrap(),
                    y2: reader.read_f32().unwrap(),
                    x3: reader.read_f32().unwrap(),
                    y3: reader.read_f32().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "BEZIER" => {
                if reader.remaining() != 4 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::BEZIER(Bezier {
                    id: reader.read_string().unwrap(),
                    control_points: reader.read_value().unwrap().as_array().clone()
                        .unwrap().windows(2).map(|a| Point2D::new(a[0].as_f64().unwrap() as f32, a[1].as_f64().unwrap() as f32)).collect(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "TEXT" => {
                if reader.remaining() != 6 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::TEXT(Text {
                    id: reader.read_string().unwrap(),
                    x: reader.read_f32().unwrap(),
                    y: reader.read_f32().unwrap(),
                    rotation: reader.read_f32().unwrap(),
                    text: reader.read_string().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.can_read() && reader.read_bool().is_some_and(|b| b),
                })))
            }
            "PIN" => {
                let param_count = reader.remaining();
                if param_count != 11 && param_count != 10 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::PIN(Pin {
                    id: reader.read_string().unwrap(),
                    display: reader.read_bool().unwrap(),
                    electric: reader.read_bool(),
                    x: reader.read_f32().unwrap(),
                    y: reader.read_f32().unwrap(),
                    length: reader.read_f32().unwrap(),
                    rotation: reader.read_f32().unwrap(),
                    pin_color: reader.read_string(),
                    pin_shape: if param_count == 10 { PinShape::None } else { reader.read_enum().unwrap() },
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "OBJ" => {
                let param_count = reader.remaining();
                if param_count != 10 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Symbol, property_type.into()));
                }

                Ok(Some(SymbolElement::OBJ(Object {
                    id: reader.read_string().unwrap(),
                    file_name: reader.read_string().unwrap(),
                    x: reader.read_f32().unwrap(),
                    y: reader.read_f32().unwrap(),
                    width: reader.read_f32().unwrap(),
                    height: reader.read_f32().unwrap(),
                    rotation: reader.read_f32().unwrap(),
                    is_mirrored: reader.read_bool().unwrap(),
                    data_url: reader.read_string().unwrap(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            _ => Err(ParserError::InvalidPropertyType(ParserType::Symbol, property_type.to_string())),
        }
    }
}