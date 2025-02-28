use crate::easyeda::geometry::Point2D;
use crate::easyeda::json_reader::JsonArrayReader;
use crate::easyeda::errors::{FootprintConverterError, ParserError, ParserType};
use crate::kicad::model::common::{Font, FontSize, Position, StrokeDefinition, TextEffect, TextJustifyHorizontal, TextJustifyVertical};
use crate::kicad::model::footprint_library::{DrillDefinition, FootprintArc, FootprintAttributes, FootprintCircle, FootprintLibrary, FootprintLine, FootprintPad, FootprintPadPrimitives, FootprintPolygon, FootprintRectangle, FootprintText, FootprintTextType, FootprintType, PadShape, PadType, PcbLayer, PrimitivesContainer, Scalar2D, Scalar3D};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::ops::Add;

#[allow(unused)]
#[derive(Debug)]
pub struct EasyEDAFootprint {
    pub head: Option<Head>,
    pub canvas: Canvas,
    pub layers: HashMap<u8, Layer>,
    pub physical_layers: HashMap<u8, PhysicalLayer>,
    pub active_layer: u8,
    pub fills: HashMap<String, Fill>,
    pub polygons: HashMap<String, Poly>,
    pub pads: HashMap<String, Pad>,
    pub attributes: Vec<Attribute>,

    pub part_number: Option<String>,
    pub nets: Vec<Net>,
    pub rule_template: Option<RuleTemplate>,
    pub rules: Vec<Rule>,
    pub primitives: Vec<Primitive>,
    pub strings: HashMap<String, StringObject>,
    pub vias: HashMap<String, Via>,
    pub images: HashMap<String, Image>,
}

impl EasyEDAFootprint {
    #[allow(unused)]
    pub fn load_and_parse(path: &str) -> anyhow::Result<EasyEDAFootprint> {
        let data = std::fs::read_to_string(path)?;
        Self::parse(&data)
    }

    pub fn parse(symbol_data: &str) -> anyhow::Result<EasyEDAFootprint> {
        let mut canvas = None;
        let mut head = None;
        let mut rule_template = None;
        let mut rules = Vec::new();
        let mut physical_layers = HashMap::new();
        let mut layers = HashMap::new();
        let mut fills = HashMap::new();
        let mut pads = HashMap::new();
        let mut vias = HashMap::new();
        let mut polygons = HashMap::new();
        let mut attributes = Vec::new();
        let mut nets = Vec::new();
        let mut primitives = Vec::new();
        let mut strings = HashMap::new();
        let mut images = HashMap::new();

        let mut active_layer = 0;

        for param in symbol_data.split_terminator(['\r', '\n']) {
            if param.len() == 0 {
                continue;
            }

            let prop = FootprintProperty::parse_line(param)?;
            if prop.is_none() {
                continue;
            }
            let prop = prop.unwrap();
            match prop {
                FootprintProperty::DOCTYPE(doctype) => {
                    assert_eq!(doctype.kind, "FOOTPRINT");
                }
                FootprintProperty::HEAD(h) => {
                    head = Some(h);
                }
                FootprintProperty::CANVAS(c) => {
                    canvas = Some(c);
                }
                FootprintProperty::LAYER(layer) => {
                    layers.insert(layer.id, layer);
                }
                FootprintProperty::LAYER_PHYS(layer) => {
                    physical_layers.insert(layer.id, layer);
                }
                FootprintProperty::ACTIVELAYER(layer) => {
                    active_layer = layer;
                }
                FootprintProperty::FILL(fill) => {
                    fills.insert(fill.id.clone(), fill);
                }
                FootprintProperty::POLY(poly) => {
                    if !poly.path.is_null() {
                        polygons.insert(poly.id.clone(), poly);
                    }
                }
                FootprintProperty::PAD(pad) => {
                    pads.insert(pad.id.clone(), pad);
                }
                FootprintProperty::VIA(via) => {
                    vias.insert(via.id.clone(), via);
                }
                FootprintProperty::NET(net) => {
                    nets.push(net);
                }
                FootprintProperty::RULE_TEMPLATE(template) => {
                    rule_template = Some(template);
                }
                FootprintProperty::RULE(rule) => {
                    rules.push(rule);
                }
                FootprintProperty::STRING(string) => {
                    strings.insert(string.id.clone(), string);
                }
                FootprintProperty::ATTR(attribute) => {
                    if attribute.parent_id.is_none() {
                        attributes.push(attribute);
                    } else if let Some(parent_id) = &attribute.parent_id {
                        if let Some(fill) = fills.get_mut(parent_id) {
                            fill.attributes.push(attribute);
                        } else if let Some(poly) = polygons.get_mut(parent_id) {
                            poly.attributes.push(attribute);
                        } else if let Some(pad) = pads.get_mut(parent_id) {
                            pad.attributes.push(attribute);
                        } else {
                            panic!("Invalid footprint attribute: {:?}", attribute);
                        }
                    }
                }
                FootprintProperty::PRIMITIVE(primitive) => {
                    primitives.push(primitive);
                }
                FootprintProperty::IMAGE(image) => {
                    images.insert(image.id.clone(), image);
                }
            }
        }

        Ok(EasyEDAFootprint {
            head,
            canvas: canvas.unwrap(),
            part_number: None,
            layers,
            physical_layers,
            active_layer,
            fills,
            polygons,
            pads,
            vias,
            nets,
            rule_template,
            rules,
            strings,
            images,
            attributes,
            primitives,
        })
    }
}

impl TryInto<FootprintLibrary> for EasyEDAFootprint {
    type Error = FootprintConverterError;

    fn try_into(self) -> Result<FootprintLibrary, Self::Error> {
        let mut footprint = FootprintLibrary {
            node_identifier: "footprint".to_string(),

            footprint_id: "test-footprint-0603".to_string(),
            version: Some(20240108),
            generator: Some("easyeda-to-kicad".into()),
            generator_version: None,
            model: None,
            edit_timestamp: None,
            attributes: Some(FootprintAttributes {
                footprint_type: FootprintType::Smd,
                exclude_from_bom: false,
                exclude_from_pos_files: false,
                board_only: false,
            }),
            lines: Vec::new(),
            arcs: Vec::new(),
            texts: Vec::new(),
            circles: Vec::new(),
            pads: Vec::new(),
            polygons: Vec::new(),
            rectangles: Vec::new(),
            zones: Vec::new(),
            description: None,
            properties: Vec::new(),
            tags: None,
            layer: PcbLayer::FCu,
            solder_mask_margin: None,
            zone_connect: None,
        };

        let default_text_effect = TextEffect {
            font: Font {
                size: FontSize { width: 1.27, height: 1.27 },
                ..Default::default()
            },
            ..Default::default()
        };

        let scale_factor = 0.0254;

        let mut max_y = f32::MIN;
        let mut min_y = f32::MAX;

        fn get_kicad_layer(layer: &Layer) -> Result<Option<PcbLayer>, FootprintConverterError> {
            match layer.layer_type.as_str() {
                "TOP_SILK" => Ok(Some(PcbLayer::FSilkS)),
                "BOT_SILK" => Ok(Some(PcbLayer::BSilkS)),
                "COMPONENT_SHAPE" |
                "DOCUMENT" |
                "OUTLINE" |
                "MECHANICAL" |
                "BOT_ASSEMBLY" |
                "TOP_ASSEMBLY" => Ok(Some(PcbLayer::FFab)),
                "COMPONENT_MARKING" => Ok(Some(PcbLayer::FSilkS)),
                "TOP_PASTE_MASK" => Ok(Some(PcbLayer::FPaste)),
                "BOT_PASTE_MASK" => Ok(Some(PcbLayer::BPaste)),
                "TOP_SOLDER_MASK" => Ok(Some(PcbLayer::FMask)),
                "BOT_SOLDER_MASK" => Ok(Some(PcbLayer::BMask)),
                "PIN_SOLDERING" |
                "PIN_FLOATING" => Ok(Some(PcbLayer::FFab)),
                "TOP" => Ok(Some(PcbLayer::FCu)),
                "BOTTOM" => Ok(Some(PcbLayer::BCu)),
                "MULTI" => Ok(None),
                "SIGNAL" => {
                    match layer.name.as_str() {
                        "Inner1" => Ok(Some(PcbLayer::In1Cu)),
                        "Inner2" => Ok(Some(PcbLayer::In2Cu)),
                        "Inner3" => Ok(Some(PcbLayer::In3Cu)),
                        "Inner4" => Ok(Some(PcbLayer::In4Cu)),
                        "Inner5" => Ok(Some(PcbLayer::In5Cu)),
                        "Inner6" => Ok(Some(PcbLayer::In6Cu)),
                        "Inner7" => Ok(Some(PcbLayer::In7Cu)),
                        "Inner8" => Ok(Some(PcbLayer::In8Cu)),
                        "Inner9" => Ok(Some(PcbLayer::In9Cu)),
                        "Inner10" => Ok(Some(PcbLayer::In10Cu)),
                        "Inner11" => Ok(Some(PcbLayer::In11Cu)),
                        "Inner12" => Ok(Some(PcbLayer::In12Cu)),
                        "Inner13" => Ok(Some(PcbLayer::In13Cu)),
                        "Inner14" => Ok(Some(PcbLayer::In14Cu)),
                        "Inner15" => Ok(Some(PcbLayer::In15Cu)),
                        "Inner16" => Ok(Some(PcbLayer::In16Cu)),
                        "Inner17" => Ok(Some(PcbLayer::In17Cu)),
                        "Inner18" => Ok(Some(PcbLayer::In18Cu)),
                        "Inner19" => Ok(Some(PcbLayer::In19Cu)),
                        "Inner20" => Ok(Some(PcbLayer::In20Cu)),
                        "Inner21" => Ok(Some(PcbLayer::In21Cu)),
                        "Inner22" => Ok(Some(PcbLayer::In22Cu)),
                        "Inner23" => Ok(Some(PcbLayer::In23Cu)),
                        "Inner24" => Ok(Some(PcbLayer::In24Cu)),
                        "Inner25" => Ok(Some(PcbLayer::In25Cu)),
                        "Inner26" => Ok(Some(PcbLayer::In26Cu)),
                        "Inner27" => Ok(Some(PcbLayer::In27Cu)),
                        "Inner28" => Ok(Some(PcbLayer::In28Cu)),
                        "Inner29" => Ok(Some(PcbLayer::In29Cu)),
                        "Inner30" => Ok(Some(PcbLayer::In30Cu)),
                        str => Err(FootprintConverterError::UnsupportedInnerLayer(str.to_string()))
                    }
                }
                str => Err(FootprintConverterError::UnsupportedLayer(format!("{:?}", str))),
            }
        }

        // Polygons
        for (_id, polygon) in &self.polygons {
            let layer = self.layers.get(&polygon.layer_id).unwrap();
            let path = polygon.path.as_array().unwrap();
            let kicad_layer = get_kicad_layer(layer)?;
            if kicad_layer.is_none() {
                continue;
            }

            let kicad_layer = kicad_layer.unwrap();
            Self::populate_footprint_shapes(path, &mut footprint, kicad_layer, polygon.width, false, None, scale_factor, None);
        }

        // Non-mechanical fills
        for (_id, fill) in &self.fills {
            let layer = self.layers.get(&fill.layer_id).unwrap();
            let path_list = fill.path.as_array().unwrap().clone();
            let kicad_layer = get_kicad_layer(layer)?;
            if kicad_layer.is_none() {
                continue;
            }

            let kicad_layer = kicad_layer.unwrap();
            Self::populate_footprint_shapes(&path_list, &mut footprint, kicad_layer, fill.width, true, None, scale_factor, None);
        }

        // Mechanical NPTH fills
        for (_id, fill) in &self.fills {
            let layer = self.layers.get(&fill.layer_id).unwrap();
            let mut path_list = fill.path.as_array().unwrap().clone();
            if !path_list.first().unwrap().is_array() {
                path_list = vec![fill.path.clone()];
            }

            if layer.layer_type != "MULTI" {
                continue;
            }

            for sub_path in path_list {
                let path = sub_path.as_array().unwrap();
                if path.get(0).unwrap().as_str().is_some_and(|s| s == "CIRCLE") {
                    let center_x = path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                    let center_y = -path.get(2).unwrap().as_f64().unwrap() as f32 * scale_factor;
                    let radius = path.get(3).unwrap().as_f64().unwrap() as f32 * scale_factor;

                    let ki_pad = FootprintPad {
                        number: "".into(),
                        pad_type: PadType::NpThruHole,
                        pad_shape: PadShape::Circle,
                        position: Position { x: center_x, y: center_y, angle: None },
                        size: Scalar2D::new("size", radius * 2.0, radius * 2.0), // todo
                        locked: false,
                        drill: Some(DrillDefinition {
                            oval: false,
                            diameter: radius * 2.0,
                            width: None,
                            offset: None,
                        }),
                        layers: {
                            let mut vec = vec![PcbLayer::FMask, PcbLayer::BMask];
                            vec.extend(PcbLayer::all_copper());
                            vec
                        },
                        property: None,
                        remove_unused_layer: None,
                        keep_end_layers: None,
                        round_rect_ratio: None,
                        chamfer_ratio: None,
                        chamfer: vec![],
                        net: None,
                        uuid: None,
                        pin_function: None,
                        pin_type: None,
                        die_length: None,
                        solder_mask_margin: None,
                        solder_paste_margin: None,
                        solder_paste_margin_ratio: None,
                        zone_connection: None,
                        clearance: None,
                        options: None,
                        primitives: None,
                    };

                    footprint.attributes.as_mut().unwrap().footprint_type = FootprintType::ThroughHole;
                    footprint.pads.push(ki_pad);
                } else {
                    let kicad_layer = PcbLayer::EdgeCuts;
                    Self::populate_footprint_shapes(path, &mut footprint, kicad_layer, 0.05, false, None, scale_factor, None);
                }
            }
        }

        // Pads [THT + SMD]
        for (_id, pad) in &self.pads {
            let layer = self.layers.get(&pad.layer_id).unwrap();
            let path = pad.path.as_ref().unwrap().as_array().unwrap();
            let kicad_layer = get_kicad_layer(layer)?;

            max_y = max_y.max(pad.center_y * scale_factor);
            min_y = min_y.min(pad.center_y * scale_factor);

            let mut ki_pad = FootprintPad {
                number: pad.num.clone(),
                pad_type: PadType::Smd,
                pad_shape: PadShape::Custom,
                position: Position {
                    x: pad.center_x * scale_factor,
                    y: -pad.center_y * scale_factor,
                    angle: Some(pad.rotation),
                },
                size: Scalar2D::new("size", 0.0, 0.0), // todo
                locked: false,
                drill: None,
                layers: match layer.layer_type.as_str() {
                    "MULTI" => {
                        let mut vec = vec![PcbLayer::FMask, PcbLayer::BMask];
                        vec.extend(PcbLayer::all_copper());
                        vec
                    }
                    _ => vec![kicad_layer.unwrap(), PcbLayer::FMask, PcbLayer::FPaste]
                },
                property: None,
                remove_unused_layer: None,
                keep_end_layers: None,
                round_rect_ratio: None,
                chamfer_ratio: None,
                chamfer: vec![],
                net: None,
                uuid: None,
                pin_function: None,
                pin_type: None,
                die_length: None,
                solder_mask_margin: pad.top_solder_expansion.or(Some(2.0)).map(|v| v * scale_factor),
                solder_paste_margin: pad.top_paste_expansion.or(Some(0.0)).map(|v| v * scale_factor).map(|v| v.max(0.0)),
                solder_paste_margin_ratio: None,
                zone_connection: None,
                clearance: None,
                options: None,
                primitives: None,
            };

            if path.len() == 4 && path.get(0).unwrap().as_str().is_some_and(|s| s == "RECT") {
                ki_pad.pad_shape = PadShape::Rect;
                ki_pad.size.x = path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                ki_pad.size.y = path.get(2).unwrap().as_f64().unwrap() as f32 * scale_factor;
            } else if path.len() == 3 && path.get(0).unwrap().as_str().is_some_and(|s| s == "ELLIPSE") {
                ki_pad.pad_shape = PadShape::Oval;
                ki_pad.size.x = path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                ki_pad.size.y = path.get(2).unwrap().as_f64().unwrap() as f32 * scale_factor;
            } else if path.len() == 3 && path.get(0).unwrap().as_str().is_some_and(|s| s == "OVAL") {
                ki_pad.pad_shape = PadShape::Oval;
                ki_pad.size.x = path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                ki_pad.size.y = path.get(2).unwrap().as_f64().unwrap() as f32 * scale_factor;
                /*if pad.rotation.abs() == 90.0 || pad.rotation.abs() == 270.0 {
                    (ki_pad.size.x, ki_pad.size.y) = (ki_pad.size.y, ki_pad.size.x);
                }*/
            } else if path.get(0).unwrap().as_str().is_some_and(|s| s == "POLY") {
                let path_data = path.get(1).unwrap().as_array().unwrap().clone();
                // let path_data = Self::parse_path_expression(path_data, scale_factor);

                ki_pad.pad_shape = PadShape::Custom;
                ki_pad.size.x = 0.01;
                ki_pad.size.y = 0.01;

                let mut pad_primitives = FootprintPadPrimitives {
                    width: Some(0.2),
                    fill: Some(true),
                    rectangles: Vec::new(),
                    circles: Vec::new(),
                    polygons: Vec::new(),
                    lines: Vec::new(),
                    arcs: Vec::new(),
                    curves: Vec::new(),
                    annotation_boxes: Vec::new(),
                };

                Self::populate_footprint_shapes(&path_data, &mut pad_primitives, PcbLayer::FCu, 0.1, true, None, scale_factor, Some(Point2D::new(-pad.center_x * scale_factor, pad.center_y * scale_factor)));
                pad_primitives.width = None;
                pad_primitives.fill = None;
                ki_pad.primitives = Some(pad_primitives);
            } else {
                return Err(FootprintConverterError::UnsupportedPadShape(format!("{:?}", pad)));
            }

            if pad.hole.as_ref().unwrap().is_null() {
                footprint.attributes.as_mut().unwrap().footprint_type = FootprintType::Smd;
            } else if let Some(hole_shape) = pad.hole.as_ref().unwrap().as_array() {
                footprint.attributes.as_mut().unwrap().footprint_type = FootprintType::ThroughHole;

                let mut hole_param1 = hole_shape.get(1).unwrap().as_f64().unwrap() as f32;
                let mut hole_param2 = hole_shape.get(2).unwrap().as_f64().unwrap() as f32;
                let hole_shape = hole_shape.get(0).unwrap().as_str().unwrap();
                assert!(hole_shape == "SLOT" || hole_shape == "ROUND", "The following THT hole shape is not supported: '{}'", hole_shape);

                if let Some(hole_rotation) = pad.hole_rotation {
                    let hole_rotation = hole_rotation.abs() as u32 % 360;
                    match hole_rotation {
                        0 | 180 => {}
                        90 | 270 => {
                            (hole_param1, hole_param2) = (hole_param2, hole_param1)
                        }
                        rot => {
                            return Err(FootprintConverterError::UnsupportedDrillRotation(rot))
                        }
                    }
                }

                ki_pad.pad_type = PadType::ThruHole;
                ki_pad.drill = Some(DrillDefinition {
                    oval: hole_shape == "SLOT",
                    offset: Some(Scalar2D::new("offset", pad.hole_offset_x * scale_factor, pad.hole_offset_y * scale_factor)),
                    width: Some(hole_param2 * scale_factor),
                    diameter: hole_param1 * scale_factor,
                });
            }

            footprint.pads.push(ki_pad);
        }

        // Vias
        for (_id, via) in &self.vias {
            let ki_pad = FootprintPad {
                number: via.name.clone(),
                pad_type: PadType::ThruHole,
                pad_shape: PadShape::Circle,
                position: Position {
                    x: via.center_x * scale_factor,
                    y: -via.center_y * scale_factor,
                    angle: None,
                },
                size: Scalar2D::new("size", via.via_diameter * scale_factor, via.via_diameter * scale_factor), // todo
                locked: false,
                drill: Some(DrillDefinition {
                    oval: false,
                    diameter: via.hole_diameter * scale_factor,
                    width: None,
                    offset: None,
                }),
                layers: {
                    let mut vec = vec![PcbLayer::FMask, PcbLayer::BMask];
                    vec.extend(PcbLayer::all_copper());
                    vec
                },
                property: None,
                remove_unused_layer: None,
                keep_end_layers: None,
                round_rect_ratio: None,
                chamfer_ratio: None,
                chamfer: vec![],
                net: None,
                uuid: None,
                pin_function: None,
                pin_type: None,
                die_length: None,
                solder_mask_margin: via.top_solder_expansion.or(Some(2.0)).map(|v| v * scale_factor),
                solder_paste_margin: None,
                solder_paste_margin_ratio: None,
                zone_connection: None,
                clearance: None,
                options: None,
                primitives: None,
            };

            footprint.pads.push(ki_pad);
        }

        // Strings
        for (_id, string) in &self.strings {
            let layer = self.layers.get(&string.layer_id).unwrap();
            let kicad_layer = get_kicad_layer(layer)?;
            if kicad_layer.is_none() {
                continue;
            }
            let kicad_layer = kicad_layer.unwrap();

            let mut text_style = default_text_effect.clone();
            text_style.font.bold = string.is_bold;
            text_style.font.italic = string.is_italic;
            text_style.font.size.width = string.font_size * scale_factor;
            text_style.font.size.height = string.font_size * scale_factor;
            (text_style.justify.justify_horizontal, text_style.justify.justify_vertical) = match string.origin as u32 {
                1 => (Some(TextJustifyHorizontal::Left), Some(TextJustifyVertical::Bottom)),
                2 | 3 => (Some(TextJustifyHorizontal::Left), Some(TextJustifyVertical::Top)),
                4 => (Some(TextJustifyHorizontal::Right), Some(TextJustifyVertical::Bottom)),
                5 | 6 => (Some(TextJustifyHorizontal::Right), Some(TextJustifyVertical::Top)),
                7 => (Some(TextJustifyHorizontal::Right), Some(TextJustifyVertical::Bottom)),
                8 | 9 | _ => (Some(TextJustifyHorizontal::Right), Some(TextJustifyVertical::Top)),
            };

            footprint.texts.push(FootprintText {
                text_type: FootprintTextType::User,
                text: string.text.clone(),
                position: Position { x: string.pos_x * scale_factor, y: -string.pos_y * scale_factor, angle: Some(string.angle) },
                unlocked: Some(true),
                layer: kicad_layer,
                hide: false,
                effects: text_style,
                uuid: None,
            });
        }

        max_y += default_text_effect.font.size.height;
        min_y -= default_text_effect.font.size.height;

        // Reference Property
        footprint.properties.push(crate::kicad::model::footprint_library::FootprintProperty {
            key: "Reference".into(),
            value: Some("Ref**".into()),
            position: Scalar3D::new("at", 0.0, -max_y, 0.0),
            layer: PcbLayer::FSilkS,
            hide: Some(false),
            unlocked: None,
            uuid: None,
            effects: default_text_effect.clone(),
        });

        // Value Property
        footprint.properties.push(crate::kicad::model::footprint_library::FootprintProperty {
            key: "Value".into(),
            value: Some("Val**".into()),
            position: Scalar3D::new("at", 0.0, -min_y, 0.0),
            layer: PcbLayer::FFab,
            hide: Some(false),
            unlocked: None,
            uuid: None,
            effects: default_text_effect.clone(),
        });

        Ok(footprint)
    }
}

#[derive(Debug)]
pub enum PathCommand {
    MoveTo { position: Point2D },
    LineTo { position: Point2D },
    ArcTo { end: Point2D, rotation: f32 },
    CenterArcTo { end: Point2D, rotation: f32 },
    Circle { center: Point2D, radius: f32 },
    Rectangle { start: Point2D, width: f32, height: f32, rotation: f32, corner_radius: f32 },
}

impl Add for Point2D {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl EasyEDAFootprint {
    fn populate_footprint_shapes(
        paths: &Vec<Value>,
        footprint: &mut impl PrimitivesContainer,
        layer: PcbLayer, stroke_width: f32,
        filled: bool,
        stroke: Option<StrokeDefinition>,
        scale_factor: f32,
        offset: Option<Point2D>,
    ) -> bool {
        if paths.len() == 0 {
            return true;
        }

        // Handle nested arrays on the top level
        if paths.iter().all(|path| path.is_array()) {
            for sub_path in paths.iter().map(|path| path.as_array().unwrap()) {
                Self::populate_footprint_shapes(sub_path, footprint, layer, stroke_width, filled, stroke.clone(), scale_factor, offset);
            }
            return true;
        }

        let path = Self::parse_path_expression(paths.clone(), scale_factor);
        let is_standalone_shape = path.iter().all(|c| match c {
            PathCommand::Circle { .. } | PathCommand::Rectangle { .. } => true,
            _ => false,
        });
        let contains_arcs = !is_standalone_shape && path.iter().any(|c| match c {
            PathCommand::ArcTo { .. } | PathCommand::CenterArcTo { .. } => true,
            _ => false,
        });
        let path = if let Some(offset) = offset {
            path.into_iter().map(|c| match c {
                PathCommand::MoveTo { position } => PathCommand::MoveTo { position: position + offset },
                PathCommand::LineTo { position } => PathCommand::MoveTo { position: position + offset },
                PathCommand::ArcTo { end, rotation } => PathCommand::ArcTo { end: end + offset, rotation },
                PathCommand::CenterArcTo { end, rotation } => PathCommand::CenterArcTo { end: end + offset, rotation },
                PathCommand::Circle { center, radius } => PathCommand::Circle { center: center + offset, radius },
                PathCommand::Rectangle { start, width, height, rotation, corner_radius } => PathCommand::Rectangle { start: start + offset, width, height, rotation, corner_radius }
            }).collect()
        } else {
            path
        };

        let mut bb_min = Point2D::new(f32::MAX, f32::MAX);
        let mut bb_max = Point2D::new(f32::MIN, f32::MIN);
        if is_standalone_shape {
            for command in path {
                Self::expand_bbox_to_shape(&command, &mut bb_min, &mut bb_max);
                match command {
                    PathCommand::Circle { center, radius } => {
                        footprint.add_circle(FootprintCircle {
                            center: Scalar2D::new("center", center.x, center.y),
                            end: Scalar2D::new("end", center.x + radius, center.y),
                            layer,
                            width: Some(stroke_width * scale_factor),
                            fill: Some(filled),
                            stroke: None,
                            uuid: None,
                            locked: false,
                        })
                    }
                    PathCommand::Rectangle { start, width, height, rotation, corner_radius } => {
                        if rotation == 0.0 && corner_radius == 0.0 {
                            footprint.add_rectangle(FootprintRectangle {
                                start: Scalar2D::new("center", start.x, start.y),
                                end: Scalar2D::new("end", start.x + width, start.y + height),
                                layer,
                                width: Some(stroke_width * scale_factor),
                                fill: Some(filled),
                                stroke: None,
                                uuid: None,
                                locked: false,
                            })
                        } else {
                            todo!("Angled rectangles or corner radii are not implemented yet")
                        }
                    }
                    PathCommand::MoveTo { .. } |
                    PathCommand::LineTo { .. } |
                    PathCommand::ArcTo { .. } |
                    PathCommand::CenterArcTo { .. } => unreachable!(),
                }
            }
        } else if !contains_arcs {
            match path.as_slice() {
                // Handle simple lines
                [PathCommand::MoveTo { position: start }, PathCommand::LineTo { position: end }] => {
                    Self::expand_bbox_to_shape(&path[0], &mut bb_min, &mut bb_max);
                    Self::expand_bbox_to_shape(&path[1], &mut bb_min, &mut bb_max);
                    footprint.add_line(FootprintLine {
                        start: Scalar2D::new("start", start.x, start.y),
                        end: Scalar2D::new("end", end.x, end.y),
                        layer,
                        width: Some(stroke_width * scale_factor),
                        uuid: None,
                        locked: false,
                        stroke: None,
                    });
                }

                // Handle polygons
                polygon => {
                    let mut points = vec![];
                    for command in polygon {
                        Self::expand_bbox_to_shape(&command, &mut bb_min, &mut bb_max);
                        match command {
                            PathCommand::MoveTo { position } => {
                                points.push(position.to_scalar_2d("xy"));
                            }
                            PathCommand::LineTo { position } => {
                                points.push(position.to_scalar_2d("xy"));
                            }
                            PathCommand::ArcTo { .. } | PathCommand::CenterArcTo { .. } => unreachable!(),
                            PathCommand::Circle { .. } | PathCommand::Rectangle { .. } => unreachable!(),
                        }
                    }

                    footprint.add_polygon(FootprintPolygon {
                        fill: Some(filled),
                        layer,
                        width: Some(stroke_width * scale_factor),
                        points,
                        stroke: None,
                        uuid: None,
                        locked: false,
                    })
                }
            }
        } else if contains_arcs {
            // println!("{:?}", path);
            match path.as_slice() {
                // Handle standalone arc
                [PathCommand::MoveTo { position: start }, PathCommand::ArcTo { end, rotation }] |
                [PathCommand::MoveTo { position: start }, PathCommand::CenterArcTo { end, rotation }] => {
                    let start = Point2D::new(start.x, start.y);
                    let end = Point2D::new(end.x, -end.y);
                    let mid = Self::get_arc_center(start, end, *rotation);
                    footprint.add_arc(FootprintArc {
                        start: Scalar2D::new("start", start.x, start.y),
                        mid: Some(Scalar2D::new("mid", mid.x, mid.y)),
                        end: Scalar2D::new("end", end.x, end.y),
                        layer,
                        width: Some(stroke_width * scale_factor),
                        angle: None,
                        stroke: None,
                        uuid: None,
                        locked: false,
                    });
                }

                // Handle polygons
                polygon => {
                    let mut points = vec![];
                    let mut last_position = Point2D::new(0.0, 0.0);
                    for command in polygon {
                        match command {
                            PathCommand::MoveTo { position } => {
                                points.push(position.to_scalar_2d("xy"));
                                last_position = position.clone();
                            }
                            PathCommand::LineTo { position } => {
                                points.push(position.to_scalar_2d("xy"));
                                last_position = position.clone();
                            }
                            PathCommand::ArcTo { end, rotation } |
                            PathCommand::CenterArcTo { end, rotation } => {
                                let end = Point2D::new(end.x, -end.y);

                                for mid in Self::interpolate_arc_points(last_position, end, -*rotation, 8.0) {
                                    points.push(mid.to_scalar_2d("xy"));
                                }

                                points.push(end.to_scalar_2d("xy"));
                                last_position = end.clone();
                            }
                            PathCommand::Circle { .. } | PathCommand::Rectangle { .. } => unreachable!(),
                        }
                    }

                    footprint.add_polygon(FootprintPolygon {
                        fill: Some(filled),
                        layer,
                        width: Some(stroke_width * scale_factor),
                        points,
                        stroke: None,
                        uuid: None,
                        locked: false,
                    })
                }
            }
        }

        true
    }

    fn parse_path_expression(mut path: Vec<Value>, scale_factor: f32) -> Vec<PathCommand> {
        // Ensure that the first element is a Move ("M") command
        if path.first().unwrap().is_f64() || path.first().unwrap().is_i64() {
            path.insert(0, Value::String("M".into()));
        }

        // Prefix all LineTo coordinates with the "L" command
        let mut line_pair_counter = 0;
        let mut is_line_segment = false;
        let mut i = 3;
        while i < path.len() {
            let value = &path[i];
            if value.is_string() {
                is_line_segment = value.as_str().unwrap() == "L";
                if is_line_segment {
                    line_pair_counter = 0;
                }
            } else if is_line_segment {
                if line_pair_counter > 0 && (line_pair_counter) % 2 == 0 {
                    path.insert(i, Value::String("L".into()));
                    line_pair_counter = 0;
                } else {
                    line_pair_counter += 1;
                }
            }
            i += 1;
        }

        // Deserialize path commands
        let mut param_iter = path.into_iter();
        let mut path = vec![];
        while let Some(command) = param_iter.next() {
            assert!(command.is_string(), "Expected a command token, got '{:?}' instead", command);
            let command = command.as_str().unwrap();
            path.push(match command {
                "M" => PathCommand::MoveTo {
                    position: Point2D::new(
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                        -param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    )
                },
                "L" => PathCommand::LineTo {
                    position: Point2D::new(
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                        -param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    )
                },
                "ARC" => PathCommand::ArcTo {
                    rotation: param_iter.next().unwrap().as_f64().unwrap() as f32,
                    end: Point2D::new(
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    ),
                },
                "CARC" => PathCommand::CenterArcTo {
                    rotation: param_iter.next().unwrap().as_f64().unwrap() as f32,
                    end: Point2D::new(
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    ),
                },
                "CIRCLE" => PathCommand::Circle {
                    center: Point2D::new(
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                        -param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    ),
                    radius: param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                },
                "R" => PathCommand::Rectangle {
                    start: Point2D::new(
                        param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                        -param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    ),
                    width: param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    height: param_iter.next().unwrap().as_f64().unwrap() as f32 * scale_factor,
                    rotation: param_iter.next().unwrap().as_f64().unwrap() as f32,
                    corner_radius: param_iter.next().map(|v| v.as_f64().unwrap() as f32 * scale_factor).unwrap_or(0.0),
                },
                str => panic!("Unsupported command found: '{}'", str),
            });
        }

        path
    }

    fn get_arc_center(start: Point2D, end: Point2D, angle: f32) -> Point2D {
        // Calculate chord midpoint
        let chord_mid = Point2D {
            x: (start.x + end.x) / 2.0,
            y: (start.y + end.y) / 2.0,
        };

        // Calculate chord length
        let chord_length = f32::sqrt(
            (end.x - start.x).powi(2) +
                (end.y - start.y).powi(2)
        );

        // Convert arc angle to radians and get central angle
        let arc_angle = angle.abs() * std::f32::consts::PI / 180.0;
        let central_angle = arc_angle / 2.0;

        // Calculate radius using r = c/(2*sin(θ/2))
        let radius = chord_length / (2.0 * central_angle.sin());

        // Calculate sagitta (height of arc from chord)
        let sagitta = radius * (1.0 - central_angle.cos());

        // Calculate perpendicular vector to chord
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        // Direction depends on angle sign
        let sign = if angle < 0.0 { -1.0 } else { 1.0 };
        let perp_x = -dy * sign;
        let perp_y = dx * sign;

        // Normalize perpendicular vector
        let perp_length = f32::sqrt(perp_x.powi(2) + perp_y.powi(2));
        let unit_perp_x = perp_x / perp_length;
        let unit_perp_y = perp_y / perp_length;

        // Calculate arc midpoint
        Point2D {
            x: chord_mid.x + unit_perp_x * sagitta,
            y: chord_mid.y + unit_perp_y * sagitta,
        }
    }

    fn get_point_on_arc(start: Point2D, end: Point2D, mut angle: f32, t: f32) -> Point2D {
        // For major arcs, flip the direction to match SVG arc behavior
        if angle.abs() > 180.0 {
            angle = -angle;
        }

        // Calculate chord properties
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let chord_length = (dx * dx + dy * dy).sqrt();
        let angle_radians = angle * PI / 180.0;

        // Calculate radius and center
        let radius = (chord_length / 2.0) / (angle_radians.abs() / 2.0).sin();

        // Find the middle point of the chord
        let mid_x = (start.x + end.x) / 2.0;
        let mid_y = (start.y + end.y) / 2.0;

        // Calculate the center point
        let direction = if angle >= 0.0 { 1.0 } else { -1.0 };
        let center_distance = (radius * radius - (chord_length * chord_length / 4.0)).sqrt();
        let normalized_dx = dx / chord_length;
        let normalized_dy = dy / chord_length;
        let center_x = mid_x - direction * center_distance * normalized_dy;
        let center_y = mid_y + direction * center_distance * normalized_dx;

        // Calculate angles relative to center
        let start_angle = (start.y - center_y).atan2(start.x - center_x);
        let end_angle = (end.y - center_y).atan2(end.x - center_x);

        // Calculate smaller angle between start and end
        let mut delta_angle = end_angle - start_angle;

        // Normalize to -2PI to 2PI range
        delta_angle = delta_angle % (2.0 * PI);

        // Convert to -PI to PI range
        if delta_angle > PI {
            delta_angle -= 2.0 * PI;
        }
        if delta_angle < -PI {
            delta_angle += 2.0 * PI;
        }

        // For major arcs, take the long way around
        if (angle >= 0.0 && angle > 180.0) || (angle < 0.0 && angle < -180.0) {
            if delta_angle >= 0.0 {
                delta_angle -= 2.0 * PI;
            } else {
                delta_angle += 2.0 * PI;
            }
        }

        // Interpolate the angle
        let interpolated_angle = start_angle + delta_angle * t;

        // Calculate final point position
        Point2D {
            x: center_x + radius * interpolated_angle.cos(),
            y: center_y + radius * interpolated_angle.sin(),
        }
    }

    fn get_arc_length(start: Point2D, end: Point2D, angle_degrees: f32) -> f32 {
        // Calculate chord length using distance formula
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let chord_length = (dx * dx + dy * dy).sqrt();

        // Convert angle to radians (using absolute value for the formula)
        let angle_radians = angle_degrees.abs() * PI / 180.0;

        // Calculate radius using formula: R = (chord length/2) / sin(angle/2)
        let radius = (chord_length / 2.0) / (angle_radians / 2.0).sin();

        // Calculate arc length using formula: L = R * angle (in radians)
        radius * angle_radians
    }

    fn interpolate_arc_points(start: Point2D, end: Point2D, angle: f32, density: f32) -> Vec<Point2D> {
        let length = Self::get_arc_length(start, end, angle);

        let num_points = (length * density).round() as usize;

        if num_points == 0 {
            return Vec::new();
        }

        (1..=num_points)
            .map(|i| {
                let t = i as f32 / (num_points + 1) as f32;
                Self::get_point_on_arc(start, end, angle, t)
            })
            .collect()
    }

    fn expand_bbox_to_shape(command: &PathCommand, min: &mut Point2D, max: &mut Point2D) {
        match command {
            PathCommand::MoveTo { position } => {
                min.x = min.x.min(position.x);
                min.y = min.y.min(position.y);
                max.x = max.x.max(position.x);
                max.y = max.y.max(position.y);
            }
            PathCommand::LineTo { position } => {
                min.x = min.x.min(position.x);
                min.y = min.y.min(position.y);
                max.x = max.x.max(position.x);
                max.y = max.y.max(position.y);
            }
            PathCommand::ArcTo { end, rotation: _ } |
            PathCommand::CenterArcTo { end, rotation: _ } => {
                min.x = min.x.min(end.x);
                min.y = min.y.min(end.y);
                max.x = max.x.max(end.x);
                max.y = max.y.max(end.y);
            }
            PathCommand::Circle { center, radius } => {
                assert!(*radius >= 0.0, "Circles with negative radius are not supported");

                min.x = min.x.min(center.x - radius);
                min.y = min.y.min(center.y - radius);
                max.x = max.x.max(center.x + radius);
                max.y = max.y.max(center.y + radius);
            }
            PathCommand::Rectangle { start, width, height, rotation, .. } => {
                assert!(*width >= 0.0 && *height >= 0.0, "Rectangles with negative sizes are not supported");

                // Convert rotation to radians
                let rotation_rad = rotation.to_radians();
                let cos_rot = rotation_rad.cos();
                let sin_rot = rotation_rad.sin();

                // Calculate all four corners of the rectangle
                let corners = [
                    start,  // top-left
                    &Point2D {  // top-right
                        x: start.x + width * cos_rot,
                        y: start.y + width * sin_rot,
                    },
                    &Point2D {  // bottom-left
                        x: start.x - height * sin_rot,
                        y: start.y + height * cos_rot,
                    },
                    &Point2D {  // bottom-right
                        x: start.x + width * cos_rot - height * sin_rot,
                        y: start.y + width * sin_rot + height * cos_rot,
                    }
                ];

                // Update bounding box to include all corners
                for corner in corners.iter() {
                    min.x = min.x.min(corner.x);
                    min.y = min.y.min(corner.y);
                    max.x = max.x.max(corner.x);
                    max.y = max.y.max(corner.y);
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocType {
    pub kind: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    pub editor_version: String,
    pub import_flag: u32,
    pub uuid: String,
    pub source: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub id: u8,
    pub layer_type: String,
    pub name: String,
    pub status: u8,
    pub active_color: String,
    pub active_transparency: f32,
    pub inactive_color: String,
    pub inactive_transparency: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicalLayer {
    pub id: u8,
    pub material: Option<String>,
    pub thickness: f32,
    pub permittivity: Option<f32>,
    pub loss_tangent: Option<f32>,
    pub is_keep_island: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fill {
    pub id: String,
    pub group_id: u32,
    pub net: String,
    pub layer_id: u8,
    pub width: f32,
    pub fill_style: u32, // ??
    pub path: Value, // TODO
    pub is_locked: bool,

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Poly {
    pub id: String,
    pub group_id: u32,
    pub net: String,
    pub layer_id: u8,
    pub width: f32,
    pub path: Value, // TODO
    pub is_locked: bool,

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Net {
    pub name: String,
    pub net_type: Option<String>,
    pub special_color: Option<String>,
    pub hide_ratline: Option<bool>,
    pub differential_name: Option<String>,
    pub equal_length_group_name: Option<Value>,
    pub is_positive_net: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleTemplate {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub rule_type: String,
    pub name: String,
    pub is_default: bool,
    pub context: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Primitive {
    pub name: String,
    pub display: bool,
    pub pick: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StringObject {
    pub id: String,
    pub group_id: u32,
    pub layer_id: u8,
    pub pos_x: f32,
    pub pos_y: f32,
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub stroke_width: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub origin: f32,
    pub angle: f32,
    pub is_reverse: bool,
    pub reverse_expansion: f32,
    pub is_mirrored: bool,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub id: String,
    pub group_id: u32,
    pub layer_id: u8,
    pub start_x: f32,
    pub start_y: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub is_mirrored: bool,
    pub path: Vec<Value>,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pad {
    pub id: String,
    pub group_id: u32,
    pub net: String,
    pub layer_id: u8,
    pub num: String,
    pub center_x: f32,
    pub center_y: f32,
    pub rotation: f32,
    pub hole: Option<Value>, // TODO
    pub path: Option<Value>, // TODO
    pub special_pad: Option<Value>, // TODO
    pub hole_offset_y: f32,
    pub hole_offset_x: f32,
    pub hole_rotation: Option<f32>,
    pub is_plated: bool,
    pub pad_type: u32,
    pub top_solder_expansion: Option<f32>,
    pub bottom_solder_expansion: Option<f32>,
    pub top_paste_expansion: Option<f32>,
    pub bottom_paste_expansion: Option<f32>,
    pub is_locked: bool,

    pub connect_mode: Option<f32>,
    pub spoke_space: Option<f32>,
    pub spoke_width: Option<f32>,
    pub spoke_angle: Option<f32>,
    pub unused_inner_layers: Option<Value>,

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Via {
    pub id: String,
    pub group_id: u32,
    pub name: String,
    pub net: String,
    pub center_x: f32,
    pub center_y: f32,
    pub hole_diameter: f32,
    pub via_diameter: f32,
    pub is_suture: bool,
    pub top_solder_expansion: Option<f32>,
    pub bottom_solder_expansion: Option<f32>,
    pub is_locked: bool,
    pub unused_inner_layers: Option<Value>,

    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub id: String,
    pub group_id: u32,
    pub parent_id: Option<String>,
    pub layer_id: u8,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub key: String,
    pub value: Option<String>,
    pub key_visible: bool,
    pub value_visible: bool,
    pub font_family: String,
    pub font_size: f32,
    pub stroke_width: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub origin: f32,
    pub angle: f32,
    pub is_reverse: bool,
    pub reverse_expansion: f32,
    pub is_mirrored: bool,
    pub is_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Canvas {
    pub origin_x: f32,
    pub origin_y: f32,
    pub unit: String,
    pub grid_size_x: f32,
    pub grid_size_y: f32,
    pub snap_size_x: f32,
    pub snap_size_y: f32,
    pub alt_snap_size_x: Option<f32>,
    pub alt_snap_size_y: Option<f32>,
    pub grid_type: Option<u32>,
    pub multi_grid_type: Option<u32>,
    pub multi_grid_ratio: Option<f32>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum FootprintProperty {
    DOCTYPE(DocType),
    HEAD(Head),
    LAYER(Layer),
    LAYER_PHYS(PhysicalLayer),
    ACTIVELAYER(u8),
    FILL(Fill),
    POLY(Poly),
    PAD(Pad),
    VIA(Via),
    NET(Net),
    RULE_TEMPLATE(RuleTemplate),
    RULE(Rule),
    PRIMITIVE(Primitive),
    STRING(StringObject),
    IMAGE(Image),
    ATTR(Attribute),
    CANVAS(Canvas),
}

impl FootprintProperty {
    pub fn parse_line(line: &str) -> Result<Option<Self>, ParserError> {
        let array: Vec<Value> = serde_json::from_str(line)?;
        let mut reader = JsonArrayReader::new(array);

        if !reader.can_read() {
            return Ok(None);
        }

        let property_type = reader.read_string()
            .ok_or_else(|| ParserError::InvalidPropertyType(ParserType::Footprint, "Invalid type".to_string()))?;

        match property_type.as_str() {
            "DOCTYPE" => {
                if reader.remaining() != 2 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::DOCTYPE(DocType {
                    kind: reader.read_string().unwrap(),
                    version: reader.read_string().unwrap(),
                })))
            }
            "HEAD" => {
                if reader.remaining() != 1 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                let parameters = reader.read_value().unwrap();

                Ok(Some(FootprintProperty::HEAD(Head {
                    editor_version: parameters["editorVersion"].as_str().unwrap().to_string(),
                    import_flag: parameters["importFlag"].as_u64().unwrap() as u32,
                    uuid: parameters["uuid"].as_str().unwrap().to_string(),
                    source: parameters["source"].as_str().unwrap().to_string(),
                    title: parameters["title"].as_str().unwrap().to_string(),
                })))
            }
            "LAYER" => {
                if reader.remaining() != 8 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::LAYER(Layer {
                    id: reader.read_u8().unwrap(),
                    layer_type: reader.read_string().unwrap(),
                    name: reader.read_string().unwrap(),
                    status: reader.read_u8().unwrap(),
                    active_color: reader.read_string().unwrap(),
                    active_transparency: reader.read_f32().unwrap(),
                    inactive_color: reader.read_string().unwrap(),
                    inactive_transparency: reader.read_f32().unwrap(),
                })))
            }
            "LAYER_PHYS" => {
                if reader.remaining() != 6 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::LAYER_PHYS(PhysicalLayer {
                    id: reader.read_u8().unwrap(),
                    material: reader.read_string(),
                    thickness: reader.read_f32().unwrap(),
                    permittivity: reader.read_f32(),
                    loss_tangent: reader.read_f32(),
                    is_keep_island: reader.read_bool().unwrap(),
                })))
            }
            "ACTIVE_LAYER" => {
                if reader.remaining() != 1 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::ACTIVELAYER(reader.read_u8().unwrap())))
            }
            "FILL" => {
                if reader.remaining() != 8 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::FILL(Fill {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    net: reader.read_string().unwrap(),
                    layer_id: reader.read_u8().unwrap(),
                    width: reader.read_f32().unwrap(),
                    fill_style: reader.read_u32().unwrap(),
                    path: reader.read_value().unwrap(),
                    is_locked: reader.read_bool().unwrap(),

                    attributes: Vec::new(),
                })))
            }
            "POLY" => {
                if reader.remaining() != 7 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::POLY(Poly {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    net: reader.read_string().unwrap(),
                    layer_id: reader.read_u8().unwrap(),
                    width: reader.read_f32().unwrap(),
                    path: reader.read_value().unwrap(),
                    is_locked: reader.read_bool().unwrap(),

                    attributes: Vec::new(),
                })))
            }
            "PAD" => {
                if reader.remaining() < 21 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                let mut pad = Pad {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    net: reader.read_string().unwrap(),
                    layer_id: reader.read_u8().unwrap(),
                    num: reader.read_string().unwrap(),
                    center_x: reader.read_f32().unwrap(),
                    center_y: reader.read_f32().unwrap(),
                    rotation: reader.read_f32().unwrap(),
                    hole: reader.read_value(),
                    path: reader.read_value(),
                    special_pad: reader.read_value(),
                    hole_offset_x: reader.read_f32().unwrap(),
                    hole_offset_y: reader.read_f32().unwrap(),
                    hole_rotation: reader.read_f32(),
                    is_plated: reader.read_bool().unwrap(),
                    pad_type: reader.read_u32().unwrap(),
                    top_solder_expansion: reader.read_f32(),
                    bottom_solder_expansion: reader.read_f32(),
                    top_paste_expansion: reader.read_f32(),
                    bottom_paste_expansion: reader.read_f32(),
                    is_locked: reader.read_bool().unwrap(),

                    connect_mode: None,
                    spoke_space: None,
                    spoke_width: None,
                    spoke_angle: None,
                    unused_inner_layers: None,

                    attributes: Vec::new(),
                };

                if reader.can_read() {
                    pad.connect_mode = reader.read_f32();
                }
                if reader.can_read() {
                    pad.spoke_space = reader.read_f32();
                }
                if reader.can_read() {
                    pad.spoke_width = reader.read_f32();
                }
                if reader.can_read() {
                    pad.spoke_angle = reader.read_f32();
                }
                if reader.can_read() {
                    pad.unused_inner_layers = Some(reader.read_value().unwrap());
                }

                Ok(Some(FootprintProperty::PAD(pad)))
            }
            "VIA" => {
                if reader.remaining() < 12 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::VIA(Via {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    name: reader.read_string().unwrap(),
                    net: reader.read_string().unwrap(),
                    center_x: reader.read_f32().unwrap(),
                    center_y: reader.read_f32().unwrap(),
                    hole_diameter: reader.read_f32().unwrap(),
                    via_diameter: reader.read_f32().unwrap(),
                    is_suture: reader.read_bool().unwrap(),
                    top_solder_expansion: reader.read_f32(),
                    bottom_solder_expansion: reader.read_f32(),
                    is_locked: reader.read_bool().unwrap(),
                    unused_inner_layers: if reader.can_read() { reader.read_value() } else { None },

                    attributes: Vec::new(),
                })))
            }
            "NET" => {
                if reader.remaining() != 7 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::NET(Net {
                    name: reader.read_string().unwrap(),
                    net_type: reader.read_string(),
                    special_color: reader.read_string(),
                    hide_ratline: reader.read_bool(),
                    differential_name: reader.read_string(),
                    equal_length_group_name: reader.read_value(),
                    is_positive_net: reader.read_bool(),
                })))
            }
            "RULE_TEMPLATE" => {
                if reader.remaining() != 1 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::RULE_TEMPLATE(RuleTemplate {
                    name: reader.read_string().unwrap(),
                })))
            }
            "RULE" => {
                if reader.remaining() != 4 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::RULE(Rule {
                    rule_type: reader.read_string().unwrap(),
                    name: reader.read_string().unwrap(),
                    is_default: reader.read_bool().unwrap(),
                    context: reader.read_value().unwrap(),
                })))
            }
            "PRIMITIVE" => {
                if reader.remaining() != 3 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::PRIMITIVE(Primitive {
                    name: reader.read_string().unwrap(),
                    display: reader.read_bool().unwrap(),
                    pick: reader.read_bool().unwrap(),
                })))
            }
            "STRING" => {
                if reader.remaining() != 17 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::STRING(StringObject {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    layer_id: reader.read_u8().unwrap(),
                    pos_x: reader.read_f32().unwrap(),
                    pos_y: reader.read_f32().unwrap(),
                    text: reader.read_string().unwrap(),
                    font_family: reader.read_string().unwrap(),
                    font_size: reader.read_f32().unwrap(),
                    stroke_width: reader.read_f32().unwrap(),
                    is_bold: reader.read_bool().unwrap(),
                    is_italic: reader.read_bool().unwrap(),
                    origin: reader.read_f32().unwrap(),
                    angle: reader.read_f32().unwrap(),
                    is_reverse: reader.read_bool().unwrap(),
                    reverse_expansion: reader.read_f32().unwrap(),
                    is_mirrored: reader.read_bool().unwrap(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "IMAGE" => {
                if reader.remaining() != 11 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::IMAGE(Image {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    layer_id: reader.read_u8().unwrap(),
                    start_x: reader.read_f32().unwrap(),
                    start_y: reader.read_f32().unwrap(),
                    width: reader.read_f32().unwrap(),
                    height: reader.read_f32().unwrap(),
                    angle: reader.read_f32().unwrap(),
                    is_mirrored: reader.read_bool().unwrap(),
                    path: reader.read_value().unwrap().as_array().unwrap().clone(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "FONT" => { Ok(None) }
            "ATTR" => {
                if reader.remaining() != 21 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                Ok(Some(FootprintProperty::ATTR(Attribute {
                    id: reader.read_string().unwrap(),
                    group_id: reader.read_u32().unwrap(),
                    parent_id: reader.read_string().and_then(|s| if s.len() != 0 { Some(s) } else { None }),
                    layer_id: reader.read_u8().unwrap(),
                    x: reader.read_f32(),
                    y: reader.read_f32(),
                    key: reader.read_string().unwrap(),
                    value: reader.read_string(),
                    key_visible: reader.read_bool().unwrap(),
                    value_visible: reader.read_bool().unwrap(),
                    font_family: reader.read_string().unwrap(),
                    font_size: reader.read_f32().unwrap(),
                    stroke_width: reader.read_f32().unwrap(),
                    is_bold: reader.read_bool().unwrap(),
                    is_italic: reader.read_bool().unwrap(),
                    origin: reader.read_f32().unwrap(),
                    angle: reader.read_f32().unwrap(),
                    is_reverse: reader.read_bool().unwrap(),
                    reverse_expansion: reader.read_f32().unwrap(),
                    is_mirrored: reader.read_bool().unwrap(),
                    is_locked: reader.read_bool().unwrap(),
                })))
            }
            "CANVAS" => {
                if reader.remaining() < 7 {
                    return Err(ParserError::InvalidArrayLength(ParserType::Footprint, property_type.into()));
                }

                let mut canvas = Canvas {
                    origin_x: reader.read_f32().unwrap(),
                    origin_y: reader.read_f32().unwrap(),
                    unit: reader.read_string().unwrap(),
                    grid_size_x: reader.read_f32().unwrap(),
                    grid_size_y: reader.read_f32().unwrap(),
                    snap_size_x: reader.read_f32().unwrap(),
                    snap_size_y: reader.read_f32().unwrap(),
                    alt_snap_size_x: None,
                    alt_snap_size_y: None,
                    grid_type: None,
                    multi_grid_type: None,
                    multi_grid_ratio: None,
                };

                if reader.can_read() {
                    canvas.alt_snap_size_x = reader.read_f32();
                }
                if reader.can_read() {
                    canvas.alt_snap_size_y = reader.read_f32();
                }
                if reader.can_read() {
                    canvas.grid_type = reader.read_u32();
                }
                if reader.can_read() {
                    canvas.multi_grid_type = reader.read_u32();
                }
                if reader.can_read() {
                    canvas.multi_grid_ratio = reader.read_f32();
                }

                Ok(Some(FootprintProperty::CANVAS(canvas)))
            }
            "RULE_SELECTOR" | "PREFERENCE" | "PANELIZE" | "PANELIZE_STAMP" | "PANELIZE_SIDE" | "SILK_OPTS" | "CONNECT" => Ok(None),
            _ => Err(ParserError::InvalidPropertyType(ParserType::Footprint, property_type.to_string())),
        }
    }
}