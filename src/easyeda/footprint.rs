use std::collections::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::easyeda::json_reader::JsonArrayReader;
use crate::easyeda::{ParserError, ParserType};
use crate::kicad::model::common::{Font, FontSize, Position, TextEffect, TextJustifyHorizontal, TextJustifyVertical};
use crate::kicad::model::footprint_library::{DrillDefinition, FootprintArc, FootprintAttributes, FootprintCircle, FootprintLibrary, FootprintLine, FootprintPad, FootprintPolygon, FootprintText, FootprintTextType, FootprintType, PadShape, PadType, PcbLayer, Scalar2D, Scalar3D};

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
}

impl EasyEDAFootprint {
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
        let mut polygons = HashMap::new();
        let mut attributes = Vec::new();
        let mut nets = Vec::new();
        let mut primitives = Vec::new();
        let mut strings = HashMap::new();

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
                    polygons.insert(poly.id.clone(), poly);
                }
                FootprintProperty::PAD(pad) => {
                    pads.insert(pad.id.clone(), pad);
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
            nets,
            rule_template,
            rules,
            strings,
            attributes,
            primitives,
        })
    }
}

impl Into<FootprintLibrary> for EasyEDAFootprint {
    fn into(self) -> FootprintLibrary {
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

        fn get_kicad_layer(layer: &Layer) -> Option<PcbLayer> {
            match layer.layer_type.as_str() {
                "TOP_SILK" => Some(PcbLayer::FSilkS),
                "COMPONENT_SHAPE" | "DOCUMENT" => Some(PcbLayer::FFab),
                "COMPONENT_MARKING" => Some(PcbLayer::FSilkS),
                "TOP_PASTE_MASK" => Some(PcbLayer::FPaste),
                "PIN_SOLDERING" | "PIN_FLOATING" => Some(PcbLayer::FFab),
                "TOP" => Some(PcbLayer::FCu),
                "MULTI" => None,
                str => panic!("Footprint elements are not supported on layer: {}", str),
            }
        }

        // Polygons
        for (_id, polygon) in &self.polygons {
            let layer = self.layers.get(&polygon.layer_id).unwrap();
            let path = polygon.path.as_array().unwrap();
            let kicad_layer = get_kicad_layer(layer);
            if kicad_layer.is_none() {
                continue;
            }
            let kicad_layer = kicad_layer.unwrap();

            if path.len() == 5 && path.get(2).unwrap().as_str().is_some_and(|s| s == "L") { // Simple line
                let start_x = path.get(0).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let start_y = -path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let end_x = path.get(3).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let end_y = -path.get(4).unwrap().as_f64().unwrap() as f32 * scale_factor;

                max_y = max_y.max(start_y.max(end_y));
                min_y = min_y.min(start_y.min(end_y));

                footprint.lines.push(FootprintLine {
                    start: Scalar2D::new("start", start_x, start_y),
                    end: Scalar2D::new("end", end_x, end_y),
                    layer: kicad_layer,
                    width: Some(polygon.width * scale_factor),
                    uuid: None,
                    locked: false,
                    stroke: None,
                });
            } else if path.len() >= 3 && path.get(2).unwrap().as_str().is_some_and(|s| s == "L") { // Hollow polygon
                let points = path
                    .iter()
                    .filter(|v| v.is_f64() || v.is_i64())
                    .collect_vec()
                    .chunks(2)
                    .into_iter()
                    .map(|c| Scalar2D::new("xy",
                                           c.get(0).unwrap().as_f64().unwrap() as f32 * scale_factor,
                                           -c.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor))
                    .collect_vec();

                points.iter().for_each(|p| {
                    max_y = max_y.max(p.y);
                    min_y = min_y.min(p.y);
                });

                footprint.polygons.push(FootprintPolygon {
                    fill: Some(false),
                    layer: kicad_layer,
                    width: Some(polygon.width * scale_factor),
                    points,
                    stroke: None,
                    uuid: None,
                    locked: false,
                })
            } else if path.len() == 4 && path.get(0).unwrap().as_str().is_some_and(|s| s == "CIRCLE") {
                let center_x = path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let center_y = -path.get(2).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let radius = path.get(3).unwrap().as_f64().unwrap() as f32 * scale_factor;

                max_y = max_y.max(center_y + radius);
                min_y = min_y.min(center_y - radius);

                footprint.circles.push(FootprintCircle {
                    center: Scalar2D::new("center", center_x, center_y),
                    end: Scalar2D::new("end", center_x + radius, center_y),
                    layer: kicad_layer,
                    width: Some(polygon.width * scale_factor),
                    stroke: None,
                    fill: Some(false),
                    uuid: None,
                    locked: false,
                });
            } else if path.len() == 6 && path.get(2).unwrap().as_str().is_some_and(|s| s == "ARC") {
                let mut start_x = path.get(0).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let mut start_y = -path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let mut end_x = path.get(4).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let mut end_y = -path.get(5).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let rotation = -path.get(3).unwrap().as_f64().unwrap() as f32;

                // Calculate the chord's midpoint
                let chord_mid_x = (start_x + end_x) / 2.0;
                let chord_mid_y = (start_y + end_y) / 2.0;

                // Calculate the chord length
                let chord_dx = end_x - start_x;
                let chord_dy = end_y - start_y;
                let chord_length = (chord_dx * chord_dx + chord_dy * chord_dy).sqrt();

                // Convert rotation to radians and get its properties
                let rotation_rad = rotation.to_radians();
                let is_major_arc = rotation.abs() > 180.0;
                let direction = if rotation >= 0.0 { 1.0 } else { -1.0 };

                // Calculate radius using the chord length and rotation angle
                // Using the formula: R = (chord_length/2) / sin(rotation/2)
                let half_rotation_rad = rotation_rad / 2.0;
                let radius = (chord_length / 2.0) / half_rotation_rad.sin().abs(); // Use abs to handle negative angles

                // Calculate the perpendicular vector to the chord
                let perp_dx = -chord_dy / chord_length;
                let perp_dy = chord_dx / chord_length;

                // Calculate the distance from chord midpoint to circle center
                // Using the formula: distance = R * cos(rotation/2)
                let center_distance = radius * half_rotation_rad.cos().abs(); // Use abs to handle negative angles

                // Calculate the circle center
                let center_x = chord_mid_x + direction * perp_dx * center_distance;
                let center_y = chord_mid_y + direction * perp_dy * center_distance;

                // Calculate the start angle
                let start_angle = (start_y - center_y).atan2(start_x - center_x);

                // For the middle point, we need to consider:
                // - For minor arcs (< 180°): We want half the actual rotation
                // - For major arcs (> 180°): We want half the complementary rotation (360° - rotation)
                let mid_angle_offset = if is_major_arc {
                    // For major arcs, we go the long way around
                    let complement = 360.0 - rotation.abs();
                    (complement / 2.0).to_radians() * direction * -1.0 // Reverse direction for major arcs
                } else {
                    // For minor arcs, we go the short way around
                    half_rotation_rad
                };

                // Calculate the middle point
                let mid_x = center_x + radius * (start_angle + mid_angle_offset).cos();
                let mid_y = center_y + radius * (start_angle + mid_angle_offset).sin();

                footprint.arcs.push(FootprintArc {
                    start: Scalar2D::new("start", start_x, start_y),
                    mid: Some(Scalar2D::new("mid", mid_x, mid_y)),
                    end: Scalar2D::new("end", end_x, end_y),
                    layer: kicad_layer,
                    width: Some(polygon.width * scale_factor),
                    angle: None,
                    stroke: None,
                    uuid: None,
                    locked: false,
                })
            } else if path.len() == 6 && path.get(2).unwrap().as_str().is_some_and(|s| s == "CARC") {
                let mut start_x = path.get(0).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let mut start_y = -path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let mut end_x = path.get(4).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let mut end_y = -path.get(5).unwrap().as_f64().unwrap() as f32 * scale_factor;
                let rotation = path.get(3).unwrap().as_f64().unwrap() as f32;

                // Calculate the chord's midpoint
                let chord_mid_x = (start_x + end_x) / 2.0;
                let chord_mid_y = (start_y + end_y) / 2.0;

                // Calculate the chord length
                let chord_dx = end_x - start_x;
                let chord_dy = end_y - start_y;
                let chord_length = (chord_dx * chord_dx + chord_dy * chord_dy).sqrt();

                // Convert rotation to radians and get its properties
                let rotation_rad = rotation.to_radians();
                let is_major_arc = rotation.abs() > 180.0;
                let direction = if rotation >= 0.0 { 1.0 } else { -1.0 };

                // Calculate radius using the chord length and rotation angle
                // Using the formula: R = (chord_length/2) / sin(rotation/2)
                let half_rotation_rad = rotation_rad / 2.0;
                let radius = (chord_length / 2.0) / half_rotation_rad.sin().abs(); // Use abs to handle negative angles

                // Calculate the perpendicular vector to the chord
                let perp_dx = -chord_dy / chord_length;
                let perp_dy = chord_dx / chord_length;

                // Calculate the distance from chord midpoint to circle center
                // Using the formula: distance = R * cos(rotation/2)
                let center_distance = radius * half_rotation_rad.cos().abs(); // Use abs to handle negative angles

                // Calculate the circle center
                let center_x = chord_mid_x + direction * perp_dx * center_distance;
                let center_y = chord_mid_y + direction * perp_dy * center_distance;

                // Calculate the start angle
                let start_angle = (start_y - center_y).atan2(start_x - center_x);

                // For the middle point, we need to consider:
                // - For minor arcs (< 180°): We want half the actual rotation
                // - For major arcs (> 180°): We want half the complementary rotation (360° - rotation)
                let mid_angle_offset = if is_major_arc {
                    // For major arcs, we go the long way around
                    let complement = 360.0 - rotation.abs();
                    (complement / 2.0).to_radians() * direction * -1.0 // Reverse direction for major arcs
                } else {
                    // For minor arcs, we go the short way around
                    half_rotation_rad
                };

                // Calculate the middle point
                let mid_x = center_x + radius * (start_angle + mid_angle_offset).cos();
                let mid_y = center_y + radius * (start_angle + mid_angle_offset).sin();

                footprint.arcs.push(FootprintArc {
                    start: Scalar2D::new("start", start_x, start_y),
                    mid: Some(Scalar2D::new("mid", mid_x, mid_y)),
                    end: Scalar2D::new("end", end_x, end_y),
                    layer: kicad_layer,
                    width: Some(polygon.width * scale_factor),
                    angle: None,
                    stroke: None,
                    uuid: None,
                    locked: false,
                })
            } else {
                panic!("This type of polygon element is not currently implemented: {:?}", polygon);
            }
        }

        // Non-mechanical fills
        for (_id, fill) in &self.fills {
            let layer = self.layers.get(&fill.layer_id).unwrap();
            let mut path_list = fill.path.as_array().unwrap().clone();
            let kicad_layer = get_kicad_layer(layer);
            if kicad_layer.is_none() {
                continue;
            }
            let kicad_layer = kicad_layer.unwrap();

            if !path_list.first().unwrap().is_array() {
                path_list = vec![fill.path.clone()];
            }

            for sub_path in path_list {
                let path = sub_path.as_array().unwrap();
                if path.len() == 4 && path.get(0).unwrap().as_str().is_some_and(|s| s == "CIRCLE") {
                    let center_x = path.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor;
                    let center_y = -path.get(2).unwrap().as_f64().unwrap() as f32 * scale_factor;
                    let radius = path.get(3).unwrap().as_f64().unwrap() as f32 * scale_factor;

                    max_y = max_y.max(center_y + radius);
                    min_y = min_y.min(center_y + radius);

                    footprint.circles.push(FootprintCircle {
                        center: Scalar2D::new("center", center_x, center_y),
                        end: Scalar2D::new("end", center_x + radius, center_y),
                        layer: kicad_layer,
                        width: Some(fill.width * scale_factor),
                        fill: Some(true),
                        stroke: None,
                        uuid: None,
                        locked: false,
                    })
                } else if path.len() > 3 && path.get(2).unwrap().as_str().is_some_and(|s| s == "L") {
                    let points = path
                        .iter()
                        .filter(|v| v.is_f64() || v.is_i64())
                        .collect_vec()
                        .chunks(2)
                        .into_iter()
                        .map(|c| Scalar2D::new("xy",
                                               c.get(0).unwrap().as_f64().unwrap() as f32 * scale_factor,
                                               -c.get(1).unwrap().as_f64().unwrap() as f32 * scale_factor))
                        .collect_vec();

                    points.iter().for_each(|p| {
                        max_y = max_y.max(p.y);
                        min_y = min_y.min(p.y);
                    });

                    footprint.polygons.push(FootprintPolygon {
                        fill: Some(true),
                        layer: kicad_layer,
                        width: Some(fill.width * scale_factor),
                        points,
                        stroke: None,
                        uuid: None,
                        locked: false,
                    })
                } else {
                    panic!("This type of fill element is not currently implemented: {:?}", fill);

                    // todo handle [0, 55, "ARC", -90,-20, 75, "L", -20, 125, 20, 125, 20, 75, "ARC", -90, 0, 55]
                }
            }
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
                    panic!("This type of fill element is not currently implemented: {:?}", fill);
                }
            }
        }

        // Pads [THT + SMD]
        for (_id, pad) in &self.pads {
            let layer = self.layers.get(&pad.layer_id).unwrap();
            let path = pad.path.as_ref().unwrap().as_array().unwrap();
            let kicad_layer = get_kicad_layer(layer);

            max_y = max_y.max(pad.center_y * scale_factor);
            min_y = min_y.min(pad.center_y * scale_factor);

            let mut ki_pad = FootprintPad {
                number: pad.num.clone(),
                pad_type: PadType::Smd,
                pad_shape: PadShape::Custom,
                position: Position { x: pad.center_x * scale_factor, y: -pad.center_y * scale_factor, angle: Some(pad.rotation) },
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
                solder_paste_margin: pad.top_paste_expansion.or(Some(0.0)).map(|v| v * scale_factor),
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
            } else {
                panic!("This type of pad shape is not currently implemented: {:?}", pad);
            }

            if pad.hole.as_ref().unwrap().is_null() {
                footprint.attributes.as_mut().unwrap().footprint_type = FootprintType::Smd;
            } else if let Some(hole_shape) = pad.hole.as_ref().unwrap().as_array() {
                footprint.attributes.as_mut().unwrap().footprint_type = FootprintType::ThroughHole;

                let mut hole_param1 = hole_shape.get(1).unwrap().as_f64().unwrap() as f32;
                let mut hole_param2 = hole_shape.get(2).unwrap().as_f64().unwrap() as f32;
                let hole_shape = hole_shape.get(0).unwrap().as_str().unwrap();
                assert!(hole_shape == "SLOT" || hole_shape == "ROUND", "The following THT hole shape is not supported: '{}'", hole_shape);

                ki_pad.pad_type = PadType::ThruHole;
                ki_pad.drill = Some(DrillDefinition {
                    oval: hole_shape == "SLOT",
                    offset: Some(Scalar2D::new("offset", pad.hole_offset_x * scale_factor, pad.hole_offset_y * scale_factor)),
                    width: Some(hole_param1 * scale_factor),
                    diameter: hole_param2 * scale_factor,
                });
            }

            footprint.pads.push(ki_pad);
        }

        // Strings
        for (_id, string) in &self.strings {
            let layer = self.layers.get(&string.layer_id).unwrap();
            let kicad_layer = get_kicad_layer(layer);
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
            println!("{}", string.origin);

            footprint.texts.push(FootprintText {
                text_type: FootprintTextType::User,
                text: string.text.clone(),
                position: Position { x: string.pos_x * scale_factor, y: string.pos_y * scale_factor, angle: Some(string.angle) },
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

        footprint
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
    NET(Net),
    RULE_TEMPLATE(RuleTemplate),
    RULE(Rule),
    PRIMITIVE(Primitive),
    STRING(StringObject),
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