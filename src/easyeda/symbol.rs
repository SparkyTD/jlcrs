use std::collections::HashMap;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::easyeda::json_reader::JsonArrayReader;
use crate::easyeda::model::{ParserError};
use crate::kicad::model::common::{Font, FontSize, Position, StrokeDefinition, TextEffect};
use crate::kicad::model::symbol_library::{Color, FillDefinition, FillType, PinElectricalType, PinGraphicStyle, Property, StrokeType, Symbol, SymbolArc, SymbolCircle, SymbolLib, SymbolLine, SymbolPin, SymbolRectangle};

pub struct EasyEDASymbol {
    pub line_styles: HashMap<String, LineStyle>,
    pub font_styles: HashMap<String, FontStyle>,
    pub part: Part,
    pub rectangles: HashMap<String, Rectangle>,
    pub circles: HashMap<String, Circle>,
    pub lines: HashMap<String, PolyLine>,
    pub arcs: HashMap<String, Arc>,
    pub pins: HashMap<String, Pin>,

    pub part_number: Option<String>,
}

impl EasyEDASymbol {
    pub fn load_and_parse(path: &str) -> anyhow::Result<EasyEDASymbol> {
        let data = std::fs::read_to_string(path)?;
        Self::parse(&data)
    }

    pub fn parse(symbol_data: &str) -> anyhow::Result<EasyEDASymbol> {
        let mut line_styles = HashMap::new();
        let mut font_styles = HashMap::new();
        let mut parts = HashMap::new();
        let mut rectangles: HashMap<String, Rectangle> = HashMap::new();
        let mut circles: HashMap<String, Circle> = HashMap::new();
        let mut lines: HashMap<String, PolyLine> = HashMap::new();
        let mut arcs: HashMap<String, Arc> = HashMap::new();
        let mut pins: HashMap<String, Pin> = HashMap::new();

        for param in symbol_data.split_terminator(['\r', '\n']) {
            if param.len() == 0 {
                continue;
            }

            let prop = SymbolProperty::parse_line(param)?;
            if prop.is_none() {
                continue;
            }
            let prop = prop.unwrap();

            match prop {
                SymbolProperty::DOCTYPE(doctype) => {
                    assert_eq!(doctype.kind, "SYMBOL");
                }
                SymbolProperty::HEAD(_) => {}
                SymbolProperty::LINESTYLE(line_style) => {
                    line_styles.insert(line_style.index_name.clone(), line_style);
                }
                SymbolProperty::FONTSTYLE(font_style) => {
                    font_styles.insert(font_style.index_name.clone(), font_style);
                }
                SymbolProperty::PART(part) => {
                    parts.insert(part.id.clone(), part);
                }
                SymbolProperty::ATTR(attribute) => {
                    if attribute.parent_id.is_none() {
                        parts.values_mut().last().unwrap().attributes.push(attribute);
                    } else if let Some(parent_id) = &attribute.parent_id {
                        if let Some(rectangle) = rectangles.get_mut(parent_id) {
                            rectangle.attributes.push(attribute);
                        } else if let Some(circle) = circles.get_mut(parent_id) {
                            circle.attributes.push(attribute);
                        } else if let Some(pin) = pins.get_mut(parent_id) {
                            pin.attributes.push(attribute);
                        } else {
                            panic!("Invalid symbol attribute: {:?}", attribute);
                        }
                    }
                }
                SymbolProperty::RECT(rect) => {
                    rectangles.insert(rect.id.clone(), rect);
                }
                SymbolProperty::CIRCLE(circle) => {
                    circles.insert(circle.id.clone(), circle);
                }
                SymbolProperty::POLYLINE(polyline) => {
                    lines.insert(polyline.id.clone(), polyline);
                }
                SymbolProperty::ARC(arc) => {
                    arcs.insert(arc.id.clone(), arc);
                }
                SymbolProperty::PIN(pin) => {
                    pins.insert(pin.id.clone(), pin);
                }
            }
        }

        Ok(Self {
            part: parts.into_values().last().unwrap(),
            part_number: None,
            line_styles,
            font_styles,
            rectangles,
            circles,
            lines,
            arcs,
            pins,
        })
    }
}

impl Into<SymbolLib> for EasyEDASymbol {
    fn into(self) -> SymbolLib {
        SymbolLib {
            version: 20211014,
            generator: "easyeda-to-kicad".into(),
            generator_version: None,
            symbols: vec![self.into()],
        }
    }
}

impl Into<Symbol> for EasyEDASymbol {
    fn into(self) -> Symbol {
        let mut kicad_symbol = Symbol {
            in_bom: true,
            on_board: true,
            ..Default::default()
        };

        let default_text_effect = TextEffect {
            font: Font {
                size: FontSize { width: 1.27, height: 1.27 },
                ..Default::default()
            },
            ..Default::default()
        };

        let scale_factor = 0.254;

        kicad_symbol.symbol_id = self.part.id.clone();

        for (_, rectangle) in self.rectangles {
            let line_style = self.line_styles.get(&rectangle.style_id.unwrap()).unwrap();
            kicad_symbol.rectangles.push(SymbolRectangle {
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

        for (_, circle) in self.circles {
            let line_style = self.line_styles.get(&circle.style_id.unwrap()).unwrap();
            kicad_symbol.circles.push(SymbolCircle {
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

        for (_, line) in self.lines {
            let line_style = self.line_styles.get(&line.style_id.unwrap()).unwrap();
            kicad_symbol.lines.push(SymbolLine {
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

        for (_, arc) in self.arcs {
            let line_style = self.line_styles.get(&arc.style_id.unwrap()).unwrap();
            kicad_symbol.arcs.push(SymbolArc {
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

        for (_, pin) in self.pins {
            let name = pin.attributes.iter().filter(|a| a.key == "NAME").last().unwrap();
            let number = pin.attributes.iter().filter(|a| a.key == "NUMBER").last().unwrap();
            kicad_symbol.pins.push(SymbolPin {
                position: Position { x: pin.x * scale_factor, y: pin.y * scale_factor, angle: Some(pin.rotation) },
                length: pin.length * scale_factor,
                number: Some(number.value.clone().unwrap()),
                name: Some(name.value.clone().unwrap()),
                name_effects: default_text_effect.clone(),
                number_effects: default_text_effect.clone(),
                graphic_style: PinGraphicStyle::Line,
                electrical_type: PinElectricalType::Unspecified,
            });
        }

        if let Some(part_number) = self.part_number {
            /*kicad_symbol.properties.push(Property {
                id: None,
                key: "LCSC".into(),
                value: part_number,
                hide: true,
                position: Position::default(),
                text_effects: default_text_effect.clone(),
            });*/
        }

        kicad_symbol
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    pub bbox_x: f32,
    pub bbox_y: f32,
    pub bbox_end_x: f32,
    pub bbox_end_y: f32,

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
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

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Circle {
    pub id: String,
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    pub style_id: Option<String>,
    pub is_locked: bool,

    pub attributes: Vec<Attribute>,
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

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SymbolProperty {
    DOCTYPE(DocType),
    HEAD(Head),
    LINESTYLE(LineStyle),
    FONTSTYLE(FontStyle),
    PART(Part),
    ATTR(Attribute),
    RECT(Rectangle),
    CIRCLE(Circle),
    POLYLINE(PolyLine),
    ARC(Arc),
    PIN(Pin),
}

impl SymbolProperty {
    pub fn parse_line(line: &str) -> Result<Option<Self>, ParserError> {
        let array: Vec<Value> = serde_json::from_str(line)?;
        let mut reader = JsonArrayReader::new(array);

        if !reader.can_read() {
            return Ok(None);
        }

        let property_type = reader.read_string()
            .ok_or_else(|| ParserError::InvalidPropertyType("Invalid type".to_string()))?;

        match property_type.as_str() {
            "DOCTYPE" => {
                if reader.remaining() != 2 {
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::DOCTYPE(DocType {
                    kind: reader.read_string().unwrap(),
                    version: reader.read_string().unwrap(),
                })))
            }
            "HEAD" => {
                if reader.remaining() != 1 {
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                let parameters = reader.read_value().unwrap();

                Ok(Some(SymbolProperty::HEAD(Head {
                    symbol_type: parameters["symbolType"].to_string().parse::<u32>().map_err(|e| ParserError::FormatError(e.to_string()))?,
                    version: parameters["version"].as_str().unwrap().to_string(),
                    origin_x: parameters["originX"].to_string().parse::<f32>().map_err(|e| ParserError::FormatError(e.to_string()))?,
                    origin_y: parameters["originY"].to_string().parse::<f32>().map_err(|e| ParserError::FormatError(e.to_string()))?,
                })))
            }
            "LINESTYLE" => {
                if reader.remaining() != 6 && reader.remaining() != 5 {
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::LINESTYLE(LineStyle {
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
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::FONTSTYLE(FontStyle {
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
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                let id = reader.read_string().unwrap();
                let bbox = reader.read_value().unwrap();
                let bbox: Vec<Value> = bbox["BBOX"].as_array().unwrap().to_vec();

                Ok(Some(SymbolProperty::PART(Part {
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
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::ATTR(Attribute {
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
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::RECT(Rectangle {
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

                    attributes: Vec::new(),
                })))
            }
            "CIRCLE" => {
                if reader.remaining() != 6 {
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::CIRCLE(Circle {
                    id: reader.read_string().unwrap(),
                    cx: reader.read_f32().unwrap(),
                    cy: reader.read_f32().unwrap(),
                    radius: reader.read_f32().unwrap(),
                    style_id: reader.read_string(),
                    is_locked: reader.read_bool().unwrap(),

                    attributes: Vec::new(),
                })))
            }
            "POLY" => {
                if reader.remaining() != 5 {
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                let id = reader.read_string().unwrap();
                let point_array = reader.read_value().unwrap();
                let point_array = point_array.as_array().unwrap();

                Ok(Some(SymbolProperty::POLYLINE(PolyLine {
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
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::ARC(Arc {
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
            "PIN" => {
                if reader.remaining() != 11 {
                    return Err(ParserError::InvalidArrayLength(property_type.into()));
                }

                Ok(Some(SymbolProperty::PIN(Pin {
                    id: reader.read_string().unwrap(),
                    display: reader.read_bool().unwrap(),
                    electric: reader.read_bool(),
                    x: reader.read_f32().unwrap(),
                    y: reader.read_f32().unwrap(),
                    length: reader.read_f32().unwrap(),
                    rotation: reader.read_f32().unwrap(),
                    pin_color: reader.read_string(),
                    pin_shape: reader.read_enum().unwrap(),
                    is_locked: reader.read_bool().unwrap(),

                    attributes: Vec::new(),
                })))
            }
            _ => Err(ParserError::InvalidPropertyType(property_type.to_string())),
        }
    }
}