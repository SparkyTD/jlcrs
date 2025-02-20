use crate::kicad::model::common::{Position, StrokeDefinition, TextEffect};
use crate::kicad::model::graphical::{GraphicAnnotationBox, GraphicArc, GraphicCircle, GraphicCurve, GraphicLine, GraphicPolygon, GraphicRectangle};
use crate::kicad::syntax::{PositionPreference, SyntaxArgument, SyntaxItem, SyntaxItemSerializable, TopLevelSerializable};
use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use std::collections::HashMap;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Debug)]
pub struct FootprintLibrary {
    pub node_identifier: String,
    pub footprint_id: String,
    pub version: Option<usize>,
    pub generator: Option<String>,
    pub generator_version: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub layer: PcbLayer,
    pub edit_timestamp: Option<DateTime<Utc>>,
    pub model: Option<FootprintModel>,
    pub attributes: Option<FootprintAttributes>,
    pub properties: Vec<FootprintProperty>,
    pub solder_mask_margin: Option<f32>,

    pub lines: Vec<FootprintLine>,
    pub polygons: Vec<FootprintPolygon>,
    pub circles: Vec<FootprintCircle>,
    pub rectangles: Vec<FootprintRectangle>,
    pub arcs: Vec<FootprintArc>,
    pub texts: Vec<FootprintText>,
    pub pads: Vec<FootprintPad>,
    pub zones: Vec<FootprintZone>,

    pub zone_connect: Option<ZoneConnectMode>,
}

#[derive(Debug)]
pub struct FootprintText {
    pub text_type: FootprintTextType,
    pub text: String,
    pub position: Position,
    pub unlocked: Option<bool>,
    pub layer: PcbLayer,
    pub hide: bool,
    pub effects: TextEffect,
    pub uuid: Option<String>,
}

#[derive(Debug)]
pub enum FootprintTextType {
    Reference,
    Value,
    User,
}

#[derive(Debug)]
pub struct FootprintLine {
    pub start: Scalar2D,
    pub end: Scalar2D,
    pub layer: PcbLayer,
    pub width: Option<f32>,
    pub stroke: Option<StrokeDefinition>,
    pub uuid: Option<String>,
    pub locked: bool,
}

#[derive(Debug)]
pub struct FootprintPolygon {
    pub points: Vec<Scalar2D>,
    pub layer: PcbLayer,
    pub width: Option<f32>,
    pub stroke: Option<StrokeDefinition>,
    pub fill: Option<bool>,
    pub uuid: Option<String>,
    pub locked: bool,
}

#[derive(Debug)]
pub struct FootprintCircle {
    pub center: Scalar2D,
    pub end: Scalar2D,
    pub layer: PcbLayer,
    pub width: Option<f32>,
    pub stroke: Option<StrokeDefinition>,
    pub fill: Option<bool>,
    pub uuid: Option<String>,
    pub locked: bool,
}

#[derive(Debug)]
pub struct FootprintArc {
    pub start: Scalar2D,
    pub mid: Option<Scalar2D>,
    pub end: Scalar2D,
    pub layer: PcbLayer,
    pub width: Option<f32>,
    pub angle: Option<f32>,
    pub stroke: Option<StrokeDefinition>,
    pub uuid: Option<String>,
    pub locked: bool,
}

#[derive(Debug)]
pub struct FootprintRectangle {
    pub start: Scalar2D,
    pub end: Scalar2D,
    pub layer: PcbLayer,
    pub width: Option<f32>,
    pub stroke: Option<StrokeDefinition>,
    pub fill: Option<bool>,
    pub uuid: Option<String>,
    pub locked: bool,
}

#[derive(Debug)]
// https://dev-docs.kicad.org/en/file-formats/sexpr-intro/index.html#_footprint_pad
pub struct FootprintPad {
    pub number: String,
    pub pad_type: PadType,
    pub pad_shape: PadShape,
    pub position: Position,
    pub locked: bool,
    pub size: Scalar2D,
    pub drill: Option<DrillDefinition>,
    pub layers: Vec<PcbLayer>,
    pub property: Option<PadProperty>,
    pub remove_unused_layer: Option<bool>,
    pub keep_end_layers: Option<bool>,
    pub round_rect_ratio: Option<f32>,
    pub chamfer_ratio: Option<f32>,
    pub chamfer: Vec<PadChamfer>,
    pub net: Option<Net>,
    pub uuid: Option<String>,
    pub pin_function: Option<String>,
    pub pin_type: Option<String>,
    pub die_length: Option<f32>,
    pub solder_mask_margin: Option<f32>,
    pub solder_paste_margin: Option<f32>,
    pub solder_paste_margin_ratio: Option<f32>,
    pub clearance: Option<f32>,
    pub zone_connection: Option<ZoneConnectMode>,
    // 25 - thermal_width
    // 26 - thermal_gap
    pub options: Option<FootprintPadOptions>,
    pub primitives: Option<FootprintPadPrimitives>,
}

#[derive(Debug)]
pub struct FootprintPadOptions {
    pub clearance: ClearanceType,
    pub anchor: AnchorType,
}

#[derive(Debug)]
pub struct FootprintPadPrimitives {
    pub lines: Vec<GraphicLine>,
    pub rectangles: Vec<GraphicRectangle>,
    pub arcs: Vec<GraphicArc>,
    pub circles: Vec<GraphicCircle>,
    pub curves: Vec<GraphicCurve>,
    pub polygons: Vec<GraphicPolygon>,
    pub annotation_boxes: Vec<GraphicAnnotationBox>,
    pub width: Option<f32>,
    pub fill: Option<bool>,
}

#[derive(Debug)]
pub enum ClearanceType {
    Outline,
    ConvexHull,
}

#[derive(Debug)]
pub enum AnchorType {
    Rect,
    Circle,
}

#[derive(Debug)]
pub enum ZoneConnectMode {
    NotConnected = 0,
    ThermalRelief = 1,
    SolidFill = 2,
}

#[derive(Debug)]
pub struct DrillDefinition {
    pub oval: bool,
    pub diameter: f32,
    pub width: Option<f32>,
    pub offset: Option<Scalar2D>,
}

#[derive(Debug)]
pub struct Net {
    pub number: usize,
    pub name: String,
}

#[derive(Debug)]
pub enum PadProperty {
    Bga,
    FiducialGlob,
    FiducialLoc,
    TestPoint,
    HeatSink,
    Castellated,
}

#[derive(Debug)]
pub enum PadChamfer {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug)]
pub enum PadType {
    ThruHole,
    Smd,
    Connect,
    NpThruHole,
}

#[derive(Debug)]
pub enum PadShape {
    Circle,
    Rect,
    Oval,
    Trapezoid,
    RoundRect,
    Custom,
}

#[derive(Debug)]
pub enum FootprintType {
    Smd,
    ThroughHole,
}

#[derive(Debug)]
pub struct FootprintModel {
    pub model_file: String,
    pub at: Option<Scalar3D>,
    pub scale: Option<Scalar3D>,
    pub rotate: Option<Scalar3D>,
    pub offset: Option<Scalar3D>,
    pub opacity: Option<f32>,
}

#[derive(Debug)]
pub struct FootprintAttributes {
    pub footprint_type: FootprintType,
    pub board_only: bool,
    pub exclude_from_pos_files: bool,
    pub exclude_from_bom: bool,
}

#[derive(Debug)]
pub struct FootprintProperty {
    pub key: String,
    pub value: Option<String>,
    pub position: Scalar3D,
    pub layer: PcbLayer,
    pub hide: Option<bool>,
    pub unlocked: Option<bool>,
    pub uuid: Option<String>,
    pub effects: TextEffect,
}

#[derive(Debug, Default)]
pub struct Scalar3D {
    identifier_name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default)]
pub struct Scalar2D {
    identifier_name: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct FootprintZone {
    pub net: u32,
    pub net_name: String,
    pub layer: Vec<PcbLayer>,
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub hatch_style: HatchStyle,
    pub hatch_pitch: f32,
    pub priority: Option<u32>,
    pub connect_pads: FootprintZoneConnectPads,
    pub min_thickness: f32,
    pub filled_areas_thickness: Option<bool>,
    pub keepout_settings: Option<FootprintZoneKeepoutSettings>,
    pub fill_settings: FootprintZoneFillSettings,
    pub coordinate_points: GraphicPolygon,
    // todo zone_fill_polygons - 14
    // todo zone_fill_segments - 15
}

#[derive(Debug)]
pub struct FootprintZoneKeepoutSettings {
    pub tracks_allowed: bool,
    pub vias_allowed: bool,
    pub pads_allowed: bool,
    pub copper_pour_allowed: bool,
    pub footprints_allowed: bool,
}

#[derive(Debug)]
pub struct FootprintZoneFillSettings {
    pub fill: Option<bool>,
    pub mode: ZoneFillMode,
    pub thermal_gap: f32,
    pub thermal_bridge_width: f32,
    pub smoothing: Option<ZoneSmoothingStyle>,
    pub radius: Option<f32>,
    pub island_removal_mode: Option<ZoneIslandRemovalMode>,
    pub island_area_min: Option<f32>,
    pub hatch_thickness: Option<f32>,
    pub hatch_gap: Option<f32>,
    pub hatch_orientation: Option<f32>,
    pub hatch_smoothing_level: Option<HatchSmoothingLevel>,
    pub hatch_smoothing_value: Option<f32>,
    pub hatch_border_algorithm: Option<HatchBorderAlgorithm>,
    pub hatch_min_hole_area: Option<f32>,
}

#[derive(Debug)]
pub struct FootprintZoneConnectPads {
    pub connection_type: Option<PadConnectionType>,
    pub clearance: f32,
}

#[derive(Debug)]
pub enum PadConnectionType {
    ThruHoleOnly,
    Full,
    No,
}

#[derive(Debug)]
pub enum HatchStyle {
    None,
    Edge,
    Full,
}

#[derive(Debug)]
pub enum HatchSmoothingLevel {
    NoSmoothing = 0,
    Fillet = 1,
    ArcMinimum = 2,
    ArcMaximum = 3,
}

#[derive(Debug)]
pub enum HatchBorderAlgorithm {
    ZoneMinimumThickness = 0,
    HatchThickness = 1,
}

#[derive(Debug)]
pub enum ZoneFillMode {
    Solid,
    Hatched,
}

#[derive(Debug)]
pub enum ZoneIslandRemovalMode {
    AlwaysRemove = 0,
    NeverRemove = 1,
    MinimumArea = 2,
}

#[derive(Debug)]
pub enum ZoneSmoothingStyle {
    Chamfer,
    Fillet,
}

#[derive(Debug, PartialEq, Clone, Copy, EnumIter)]
pub enum PcbLayer {
    FCu,       // Front copper layer
    In1Cu,     // Inner copper layer 1
    In2Cu,     // Inner copper layer 2
    In3Cu,     // Inner copper layer 3
    In4Cu,     // Inner copper layer 4
    In5Cu,     // Inner copper layer 5
    In6Cu,     // Inner copper layer 6
    In7Cu,     // Inner copper layer 7
    In8Cu,     // Inner copper layer 8
    In9Cu,     // Inner copper layer 9
    In10Cu,    // Inner copper layer 10
    In11Cu,    // Inner copper layer 11
    In12Cu,    // Inner copper layer 12
    In13Cu,    // Inner copper layer 13
    In14Cu,    // Inner copper layer 14
    In15Cu,    // Inner copper layer 15
    In16Cu,    // Inner copper layer 16
    In17Cu,    // Inner copper layer 17
    In18Cu,    // Inner copper layer 18
    In19Cu,    // Inner copper layer 19
    In20Cu,    // Inner copper layer 20
    In21Cu,    // Inner copper layer 21
    In22Cu,    // Inner copper layer 22
    In23Cu,    // Inner copper layer 23
    In24Cu,    // Inner copper layer 24
    In25Cu,    // Inner copper layer 25
    In26Cu,    // Inner copper layer 26
    In27Cu,    // Inner copper layer 27
    In28Cu,    // Inner copper layer 28
    In29Cu,    // Inner copper layer 29
    In30Cu,    // Inner copper layer 30
    BCu,       // Back copper layer
    BAdhes,    // Back adhesive layer
    FAdhes,    // Front adhesive layer
    BPaste,    // Back solder paste layer
    FPaste,    // Front solder paste layer
    BSilkS,    // Back silk screen layer
    FSilkS,    // Front silk screen layer
    BMask,     // Back solder mask layer
    FMask,     // Front solder mask layer
    DwgsUser,  // User drawing layer
    CmtsUser,  // User comment layer
    Eco1User,  // User engineering change order layer 1
    Eco2User,  // User engineering change order layer 2
    EdgeCuts,  // Board outline layer
    FCrtYd,    // Footprint front courtyard layer
    BCrtYd,    // Footprint back courtyard layer
    FFab,      // Footprint front fabrication layer
    BFab,      // Footprint back fabrication layer
    User1,     // User definable layer 1
    User2,     // User definable layer 2
    User3,     // User definable layer 3
    User4,     // User definable layer 4
    User5,     // User definable layer 5
    User6,     // User definable layer 6
    User7,     // User definable layer 7
    User8,     // User definable layer 8
    User9,     // User definable layer 9
}

impl PcbLayer {
    pub fn parse(str: &str) -> PcbLayer {
        match str {
            "F.Cu" => PcbLayer::FCu,
            "In1.Cu" => PcbLayer::In1Cu,
            "In2.Cu" => PcbLayer::In2Cu,
            "In3.Cu" => PcbLayer::In3Cu,
            "In4.Cu" => PcbLayer::In4Cu,
            "In5.Cu" => PcbLayer::In5Cu,
            "In6.Cu" => PcbLayer::In6Cu,
            "In7.Cu" => PcbLayer::In7Cu,
            "In8.Cu" => PcbLayer::In8Cu,
            "In9.Cu" => PcbLayer::In9Cu,
            "In10.Cu" => PcbLayer::In10Cu,
            "In11.Cu" => PcbLayer::In11Cu,
            "In12.Cu" => PcbLayer::In12Cu,
            "In13.Cu" => PcbLayer::In13Cu,
            "In14.Cu" => PcbLayer::In14Cu,
            "In15.Cu" => PcbLayer::In15Cu,
            "In16.Cu" => PcbLayer::In16Cu,
            "In17.Cu" => PcbLayer::In17Cu,
            "In18.Cu" => PcbLayer::In18Cu,
            "In19.Cu" => PcbLayer::In19Cu,
            "In20.Cu" => PcbLayer::In20Cu,
            "In21.Cu" => PcbLayer::In21Cu,
            "In22.Cu" => PcbLayer::In22Cu,
            "In23.Cu" => PcbLayer::In23Cu,
            "In24.Cu" => PcbLayer::In24Cu,
            "In25.Cu" => PcbLayer::In25Cu,
            "In26.Cu" => PcbLayer::In26Cu,
            "In27.Cu" => PcbLayer::In27Cu,
            "In28.Cu" => PcbLayer::In28Cu,
            "In29.Cu" => PcbLayer::In29Cu,
            "In30.Cu" => PcbLayer::In30Cu,
            "B.Cu" => PcbLayer::BCu,
            "B.Adhes" => PcbLayer::BAdhes,
            "F.Adhes" => PcbLayer::FAdhes,
            "B.Paste" => PcbLayer::BPaste,
            "F.Paste" => PcbLayer::FPaste,
            "B.SilkS" => PcbLayer::BSilkS,
            "F.SilkS" => PcbLayer::FSilkS,
            "B.Mask" => PcbLayer::BMask,
            "F.Mask" => PcbLayer::FMask,
            "Dwgs.User" => PcbLayer::DwgsUser,
            "Cmts.User" => PcbLayer::CmtsUser,
            "Eco1.User" => PcbLayer::Eco1User,
            "Eco2.User" => PcbLayer::Eco2User,
            "Edge.Cuts" => PcbLayer::EdgeCuts,
            "F.CrtYd" => PcbLayer::FCrtYd,
            "B.CrtYd" => PcbLayer::BCrtYd,
            "F.Fab" => PcbLayer::FFab,
            "B.Fab" => PcbLayer::BFab,
            "User.1" => PcbLayer::User1,
            "User.2" => PcbLayer::User2,
            "User.3" => PcbLayer::User3,
            "User.4" => PcbLayer::User4,
            "User.5" => PcbLayer::User5,
            "User.6" => PcbLayer::User6,
            "User.7" => PcbLayer::User7,
            "User.8" => PcbLayer::User8,
            "User.9" => PcbLayer::User9,
            _ => panic!("Invalid PcbLayer cannot be parsed: '{}'", str),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PcbLayer::FCu => "F.Cu".to_string(),
            PcbLayer::In1Cu => "In1.Cu".to_string(),
            PcbLayer::In2Cu => "In2.Cu".to_string(),
            PcbLayer::In3Cu => "In3.Cu".to_string(),
            PcbLayer::In4Cu => "In4.Cu".to_string(),
            PcbLayer::In5Cu => "In5.Cu".to_string(),
            PcbLayer::In6Cu => "In6.Cu".to_string(),
            PcbLayer::In7Cu => "In7.Cu".to_string(),
            PcbLayer::In8Cu => "In8.Cu".to_string(),
            PcbLayer::In9Cu => "In9.Cu".to_string(),
            PcbLayer::In10Cu => "In10.Cu".to_string(),
            PcbLayer::In11Cu => "In11.Cu".to_string(),
            PcbLayer::In12Cu => "In12.Cu".to_string(),
            PcbLayer::In13Cu => "In13.Cu".to_string(),
            PcbLayer::In14Cu => "In14.Cu".to_string(),
            PcbLayer::In15Cu => "In15.Cu".to_string(),
            PcbLayer::In16Cu => "In16.Cu".to_string(),
            PcbLayer::In17Cu => "In17.Cu".to_string(),
            PcbLayer::In18Cu => "In18.Cu".to_string(),
            PcbLayer::In19Cu => "In19.Cu".to_string(),
            PcbLayer::In20Cu => "In20.Cu".to_string(),
            PcbLayer::In21Cu => "In21.Cu".to_string(),
            PcbLayer::In22Cu => "In22.Cu".to_string(),
            PcbLayer::In23Cu => "In23.Cu".to_string(),
            PcbLayer::In24Cu => "In24.Cu".to_string(),
            PcbLayer::In25Cu => "In25.Cu".to_string(),
            PcbLayer::In26Cu => "In26.Cu".to_string(),
            PcbLayer::In27Cu => "In27.Cu".to_string(),
            PcbLayer::In28Cu => "In28.Cu".to_string(),
            PcbLayer::In29Cu => "In29.Cu".to_string(),
            PcbLayer::In30Cu => "In30.Cu".to_string(),
            PcbLayer::BCu => "B.Cu".to_string(),
            PcbLayer::BAdhes => "B.Adhes".to_string(),
            PcbLayer::FAdhes => "F.Adhes".to_string(),
            PcbLayer::BPaste => "B.Paste".to_string(),
            PcbLayer::FPaste => "F.Paste".to_string(),
            PcbLayer::BSilkS => "B.SilkS".to_string(),
            PcbLayer::FSilkS => "F.SilkS".to_string(),
            PcbLayer::BMask => "B.Mask".to_string(),
            PcbLayer::FMask => "F.Mask".to_string(),
            PcbLayer::DwgsUser => "Dwgs.User".to_string(),
            PcbLayer::CmtsUser => "Cmts.User".to_string(),
            PcbLayer::Eco1User => "Eco1.User".to_string(),
            PcbLayer::Eco2User => "Eco2.User".to_string(),
            PcbLayer::EdgeCuts => "Edge.Cuts".to_string(),
            PcbLayer::FCrtYd => "F.CrtYd".to_string(),
            PcbLayer::BCrtYd => "B.CrtYd".to_string(),
            PcbLayer::FFab => "F.Fab".to_string(),
            PcbLayer::BFab => "B.Fab".to_string(),
            PcbLayer::User1 => "User.1".to_string(),
            PcbLayer::User2 => "User.2".to_string(),
            PcbLayer::User3 => "User.3".to_string(),
            PcbLayer::User4 => "User.4".to_string(),
            PcbLayer::User5 => "User.5".to_string(),
            PcbLayer::User6 => "User.6".to_string(),
            PcbLayer::User7 => "User.7".to_string(),
            PcbLayer::User8 => "User.8".to_string(),
            PcbLayer::User9 => "User.9".to_string(),
        }
    }

    pub fn from(item: &SyntaxItem) -> PcbLayer {
        let str = item.arguments.first().unwrap().get_string();
        Self::parse(&str)
    }

    pub fn all_copper() -> Vec<PcbLayer> {
        vec![
            PcbLayer::FCu,
            PcbLayer::In1Cu,
            PcbLayer::In2Cu,
            PcbLayer::In3Cu,
            PcbLayer::In4Cu,
            PcbLayer::In5Cu,
            PcbLayer::In6Cu,
            PcbLayer::In7Cu,
            PcbLayer::In8Cu,
            PcbLayer::In9Cu,
            PcbLayer::In10Cu,
            PcbLayer::In11Cu,
            PcbLayer::In12Cu,
            PcbLayer::In13Cu,
            PcbLayer::In14Cu,
            PcbLayer::In15Cu,
            PcbLayer::In16Cu,
            PcbLayer::In17Cu,
            PcbLayer::In18Cu,
            PcbLayer::In19Cu,
            PcbLayer::In20Cu,
            PcbLayer::In21Cu,
            PcbLayer::In22Cu,
            PcbLayer::In23Cu,
            PcbLayer::In24Cu,
            PcbLayer::In25Cu,
            PcbLayer::In26Cu,
            PcbLayer::In27Cu,
            PcbLayer::In28Cu,
            PcbLayer::In29Cu,
            PcbLayer::In30Cu,
            PcbLayer::BCu,
        ]
    }
}

impl SyntaxItemSerializable for FootprintLibrary {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
        ];

        if let Some(tags) = &self.tags {
            children.push(SyntaxItem::from_single_argument("tags", SyntaxArgument::QuotedString(tags.clone(), PositionPreference::None)));
        }
        if let Some(description) = &self.description {
            children.push(SyntaxItem::from_single_argument("descr", SyntaxArgument::QuotedString(description.clone(), PositionPreference::None)));
        }

        if let Some(attributes) = &self.attributes {
            children.push(attributes.serialize());
        }

        if let Some(edit_timestamp) = &self.edit_timestamp {
            let ts_hex = format!("{:X}", edit_timestamp.timestamp());
            children.push(SyntaxItem::from_single_argument("tedit", SyntaxArgument::Identifier(ts_hex.to_string(), PositionPreference::None)));
        }

        if let Some(model) = &self.model {
            children.push(model.serialize());
        }
        if let Some(version) = &self.version {
            children.push(SyntaxItem::from_single_argument("version", SyntaxArgument::Identifier(version.to_string(), PositionPreference::None)));
        }
        if let Some(generator) = &self.generator {
            children.push(SyntaxItem::from_single_argument("generator", SyntaxArgument::Identifier(generator.clone(), PositionPreference::None)));
        }
        if let Some(generator_version) = &self.generator_version {
            children.push(SyntaxItem::from_single_argument("generator_version", SyntaxArgument::Identifier(generator_version.clone(), PositionPreference::None)));
        }

        children.extend(self.texts.iter().map(|item| item.serialize()));
        children.extend(self.lines.iter().map(|item| item.serialize()));
        children.extend(self.arcs.iter().map(|item| item.serialize()));
        children.extend(self.polygons.iter().map(|item| item.serialize()));
        children.extend(self.circles.iter().map(|item| item.serialize()));
        children.extend(self.rectangles.iter().map(|item| item.serialize()));
        children.extend(self.pads.iter().map(|item| item.serialize()));
        children.extend(self.zones.iter().map(|item| item.serialize()));
        children.extend(self.properties.iter().map(|item| item.serialize()));

        if let Some(solder_mask_margin) = &self.solder_mask_margin {
            children.push(SyntaxItem::from_single_argument("solder_mask_margin", SyntaxArgument::Number(*solder_mask_margin, PositionPreference::None)));
        }
        if let Some(zone_connect) = &self.zone_connect {
            children.push(SyntaxItem::from_single_argument("zone_connect", SyntaxArgument::Number(match zone_connect {
                ZoneConnectMode::NotConnected => 0,
                ZoneConnectMode::ThermalRelief => 1,
                ZoneConnectMode::SolidFill => 2,
            } as f32, PositionPreference::None)));
        }

        SyntaxItem {
            name: self.node_identifier.clone(),
            arguments: vec![SyntaxArgument::QuotedString(self.footprint_id.clone(), PositionPreference::None)],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let footprint_id = syntax.arguments.first().unwrap().get_string();

        let mut library = FootprintLibrary {
            node_identifier: syntax.name.clone(),
            footprint_id,
            version: None,
            generator: None,
            generator_version: None,
            description: None,
            tags: None,
            layer: PcbLayer::FCu,
            edit_timestamp: None,
            model: None,
            attributes: None,
            lines: Vec::new(),
            arcs: Vec::new(),
            texts: Vec::new(),
            polygons: Vec::new(),
            circles: Vec::new(),
            rectangles: Vec::new(),
            pads: Vec::new(),
            zones: Vec::new(),
            properties: Vec::new(),
            solder_mask_margin: None,
            zone_connect: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "layer" => library.layer = PcbLayer::from(&child),
                "descr" => library.description = Some(child.arguments.first().unwrap().get_string()),
                "tags" => library.tags = Some(child.arguments.first().unwrap().get_string()),
                "version" => library.version = Some(child.arguments.first().unwrap().get_number() as usize),
                "generator" => library.generator = Some(child.arguments.first().unwrap().get_string()),
                "generator_version" => library.generator_version = Some(child.arguments.first().unwrap().get_string()),
                "tedit" => library.edit_timestamp = Some(Utc.timestamp_opt(i64::from_str_radix(child.arguments.first().unwrap().get_string().as_str(), 16).unwrap(), 0).unwrap()),

                "fp_line" => library.lines.push(FootprintLine::deserialize(child)),
                "fp_arc" => library.arcs.push(FootprintArc::deserialize(child)),
                "fp_text" => library.texts.push(FootprintText::deserialize(child)),
                "fp_poly" => library.polygons.push(FootprintPolygon::deserialize(child)),
                "fp_circle" => library.circles.push(FootprintCircle::deserialize(child)),
                "fp_rect" => library.rectangles.push(FootprintRectangle::deserialize(child)),
                "zone" => library.zones.push(FootprintZone::deserialize(child)),
                "zone_connect" => library.zone_connect = Some(child.get_named_child("zone_connection").map(|s| match s.arguments.first().unwrap().get_number() as u8 {
                    0 => ZoneConnectMode::NotConnected,
                    1 => ZoneConnectMode::ThermalRelief,
                    2 => ZoneConnectMode::SolidFill,
                    num => panic!("Unsupported zone connect mode: {}", num),
                })).unwrap(),
                "pad" => library.pads.push(FootprintPad::deserialize(child)),
                "model" => { library.model.replace(FootprintModel::deserialize(child)); }
                "attr" => { library.attributes.replace(FootprintAttributes::deserialize(child)); }
                "property" => library.properties.push(FootprintProperty::deserialize(child)),

                "solder_mask_margin" => library.solder_mask_margin = Some(child.arguments.first().unwrap().get_number()),

                _ => panic!("Unsupported child item type in Footprint: {}", child.name),
            }
        }

        library
    }
}

impl SyntaxItemSerializable for FootprintAttributes {
    fn serialize(&self) -> SyntaxItem {
        let mut arguments = vec![SyntaxArgument::Identifier(match self.footprint_type {
            FootprintType::Smd => "smd",
            FootprintType::ThroughHole => "through_hole",
        }.into(), PositionPreference::Start)];

        if self.board_only {
            arguments.push(SyntaxArgument::Identifier("board_only".to_string(), PositionPreference::None));
        }
        if self.exclude_from_pos_files {
            arguments.push(SyntaxArgument::Identifier("exclude_from_pos_files".to_string(), PositionPreference::None));
        }
        if self.exclude_from_bom {
            arguments.push(SyntaxArgument::Identifier("exclude_from_bom".to_string(), PositionPreference::None));
        }

        SyntaxItem {
            name: "attr".into(),
            children: vec![],
            arguments,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut attributes = Self {
            footprint_type: match syntax.arguments.first().unwrap().get_string().as_str() {
                "smd" => FootprintType::Smd,
                "through_hole" => FootprintType::ThroughHole,
                str => panic!("Invalid footprint type in attributes: {}", str),
            },
            board_only: false,
            exclude_from_bom: false,
            exclude_from_pos_files: false,
        };

        for argument in syntax.arguments.iter().skip(1) {
            if argument.get_string() == "board_only" {
                attributes.board_only = true;
            } else if argument.get_string() == "exclude_from_bom" {
                attributes.exclude_from_bom = true;
            } else if argument.get_string() == "exclude_from_pos_files" {
                attributes.exclude_from_pos_files = true;
            }
        }

        attributes
    }
}

impl SyntaxItemSerializable for FootprintProperty {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![];
        children.push(self.position.serialize());
        children.push(SyntaxItem::from_single_argument("layer", SyntaxArgument::QuotedString(self.layer.to_string(), PositionPreference::None)));

        if let Some(hide) = self.hide {
            children.push(SyntaxItem::from_single_argument("hide", SyntaxArgument::Identifier((match hide {
                true => "yes",
                false => "no",
            }).into(), PositionPreference::None)));
        }
        if let Some(unlocked) = self.unlocked {
            children.push(SyntaxItem::from_single_argument("unlocked", SyntaxArgument::Identifier((match unlocked {
                true => "yes",
                false => "no",
            }).into(), PositionPreference::None)));
        }
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        children.push(self.effects.serialize());

        SyntaxItem {
            name: "property".into(),
            arguments: vec![
                SyntaxArgument::QuotedString(self.key.clone(), PositionPreference::Start),
                SyntaxArgument::QuotedString(self.value.clone().unwrap_or("".into()), PositionPreference::None),
            ],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut property = Self {
            key: syntax.arguments.get(0).unwrap().get_string(),
            value: match syntax.arguments.get(1).unwrap().get_string().as_str() {
                "" => None,
                str => Some(str.into()),
            },
            hide: None,
            unlocked: None,
            position: Scalar3D::default(),
            layer: PcbLayer::FCu,
            uuid: None,
            effects: TextEffect::default(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "at" => property.position = Scalar3D::deserialize(child),
                "layer" => property.layer = PcbLayer::parse(child.arguments.first().unwrap().get_string().as_str()),
                "hide" => property.hide = Some(child.arguments.get(0).is_some_and(|a| a.get_string() == "yes")),
                "unlocked" => property.unlocked = Some(child.arguments.get(0).is_some_and(|a| a.get_string() == "yes")),
                "uuid" => property.uuid = Some(child.arguments.get(0).unwrap().get_string()),
                "effects" => property.effects = TextEffect::deserialize(child),
                str => panic!("Unsupported child item type in FootprintProperty: {}", str),
            }
        }

        property
    }
}

impl SyntaxItemSerializable for FootprintModel {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![];
        if let Some(at) = &self.at {
            children.push(SyntaxItem::from_single_child("at", at.serialize()));
        }
        if let Some(scale) = &self.scale {
            children.push(SyntaxItem::from_single_child("scale", scale.serialize()));
        }
        if let Some(rotate) = &self.rotate {
            children.push(SyntaxItem::from_single_child("rotate", rotate.serialize()));
        }
        if let Some(offset) = &self.offset {
            children.push(SyntaxItem::from_single_child("offset", offset.serialize()));
        }
        if let Some(opacity) = &self.opacity {
            children.push(SyntaxItem::from_single_argument("opacity", SyntaxArgument::Number(*opacity, PositionPreference::None)));
        }

        SyntaxItem {
            name: "model".into(),
            arguments: vec![SyntaxArgument::Identifier(self.model_file.clone(), PositionPreference::None)],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut model = Self {
            model_file: syntax.arguments.first().unwrap().get_string(),
            at: None,
            scale: None,
            rotate: None,
            offset: None,
            opacity: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "at" => model.at = Some(Scalar3D::deserialize(child.get_named_child("xyz").unwrap())),
                "scale" => model.scale = Some(Scalar3D::deserialize(child.get_named_child("xyz").unwrap())),
                "rotate" => model.rotate = Some(Scalar3D::deserialize(child.get_named_child("xyz").unwrap())),
                "offset" => model.offset = Some(Scalar3D::deserialize(child.get_named_child("xyz").unwrap())),
                "opacity" => model.opacity = Some(child.arguments.first().unwrap().get_number()),
                _ => panic!("Unsupported child item type in FootprintModel: {}", child.name),
            }
        }

        model
    }
}

impl SyntaxItemSerializable for Scalar3D {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: self.identifier_name.clone(),
            children: Vec::new(),
            arguments: vec![
                SyntaxArgument::Number(self.x, PositionPreference::None),
                SyntaxArgument::Number(self.y, PositionPreference::None),
                SyntaxArgument::Number(self.z, PositionPreference::None),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            identifier_name: syntax.name.clone(),
            x: syntax.arguments.get(0).unwrap().get_number(),
            y: syntax.arguments.get(1).unwrap().get_number(),
            z: syntax.arguments.get(2).unwrap().get_number(),
        }
    }
}

impl Scalar2D {
    pub fn new(identifier: &str, x: f32, y: f32) -> Self {
        Self {
            identifier_name: identifier.to_string(),
            x,
            y,
        }
    }
}

impl Scalar3D {
    pub fn new(identifier: &str, x: f32, y: f32, z: f32) -> Self {
        Self {
            identifier_name: identifier.to_string(),
            x,
            y,
            z,
        }
    }
}

impl SyntaxItemSerializable for Scalar2D {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: self.identifier_name.clone(),
            children: Vec::new(),
            arguments: vec![
                SyntaxArgument::Number(self.x, PositionPreference::None),
                SyntaxArgument::Number(self.y, PositionPreference::None),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            identifier_name: syntax.name.clone(),
            x: syntax.arguments.get(0).unwrap().get_number(),
            y: syntax.arguments.get(1).unwrap().get_number(),
        }
    }
}

impl SyntaxItemSerializable for FootprintLine {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.start.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
        ];

        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(width, PositionPreference::None)));
        }

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        if let Some(stroke) = &self.stroke {
            children.push(stroke.serialize());
        }

        SyntaxItem {
            name: "fp_line".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut line = Self {
            layer: PcbLayer::FCu,
            start: Scalar2D::default(),
            end: Scalar2D::default(),
            width: None,
            stroke: syntax.get_named_child("stroke").and_then(|child| Some(StrokeDefinition::deserialize(child))),
            uuid: syntax.get_named_child("uuid").and_then(|child| Some(child.arguments.first().unwrap().get_string())),
            locked: syntax.get_named_child("locked").is_some(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "layer" => line.layer = PcbLayer::from(&child),
                "start" => line.start = Scalar2D::deserialize(child),
                "end" => line.end = Scalar2D::deserialize(child),
                "width" => line.width = Some(child.arguments.get(0).unwrap().get_number()),
                "locked" => line.locked = true,
                "uuid" => line.uuid = child.arguments.first().and_then(|a| Some(a.get_string())),
                "stroke" => line.stroke = Some(StrokeDefinition::deserialize(child)),
                _ => panic!("Unsupported child item type in FootprintLine: {}", child.name),
            }
        }

        line
    }
}

impl SyntaxItemSerializable for FootprintPolygon {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
        ];

        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(width, PositionPreference::None)));
        }

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        if let Some(stroke) = &self.stroke {
            children.push(stroke.serialize());
        }

        if let Some(fill) = &self.fill {
            children.push(SyntaxItem::from_single_argument("fill", SyntaxArgument::Identifier((if *fill { "yes" } else { "no" }).into(), PositionPreference::None)));
        }

        children.insert(0, SyntaxItem {
            name: "pts".into(),
            arguments: vec![],
            children: self.points.iter().map(|point| point.serialize()).collect(),
        });

        SyntaxItem {
            name: "fp_poly".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut poly = Self {
            layer: PcbLayer::FCu,
            points: Vec::new(),
            width: None,
            fill: None,
            stroke: syntax.get_named_child("stroke").and_then(|child| Some(StrokeDefinition::deserialize(child))),
            uuid: syntax.get_named_child("uuid").and_then(|child| Some(child.arguments.first().unwrap().get_string())),
            locked: syntax.get_named_child("locked").is_some(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "pts" => poly.points = child.children.iter().map(|c| Scalar2D::deserialize(c)).collect(),
                "layer" => poly.layer = PcbLayer::from(&child),
                "width" => poly.width = Some(child.arguments.get(0).unwrap().get_number()),
                "fill" => poly.fill = child.arguments.get(0).and_then(|s| Some(s.get_string() == "yes" || s.get_string() == "solid")),
                "stroke" => poly.stroke = Some(StrokeDefinition::deserialize(child)),
                "locked" => poly.locked = true,
                "uuid" => poly.uuid = child.arguments.first().and_then(|a| Some(a.get_string())),
                _ => panic!("Unsupported child item type in FootprintPolygon: {}", child.name),
            }
        }

        poly
    }
}

impl SyntaxItemSerializable for FootprintCircle {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.center.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
        ];

        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(width, PositionPreference::None)));
        }

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        if let Some(stroke) = &self.stroke {
            children.push(stroke.serialize());
        }

        if let Some(fill) = &self.fill {
            children.push(SyntaxItem::from_single_argument("fill", SyntaxArgument::Identifier((if *fill { "yes" } else { "no" }).into(), PositionPreference::None)));
        }

        SyntaxItem {
            name: "fp_circle".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut circle = Self {
            layer: PcbLayer::FCu,
            center: Scalar2D::default(),
            end: Scalar2D::default(),
            width: None,
            fill: None,
            stroke: syntax.get_named_child("stroke").and_then(|child| Some(StrokeDefinition::deserialize(child))),
            uuid: syntax.get_named_child("uuid").and_then(|child| Some(child.arguments.first().unwrap().get_string())),
            locked: syntax.get_named_child("locked").is_some(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "layer" => circle.layer = PcbLayer::from(&child),
                "center" => circle.center = Scalar2D::deserialize(child),
                "end" => circle.end = Scalar2D::deserialize(child),
                "width" => circle.width = Some(child.arguments.get(0).unwrap().get_number()),
                "fill" => circle.fill = child.arguments.get(0).and_then(|s| Some(s.get_string() == "yes" || s.get_string() == "filled")),
                "stroke" => circle.stroke = Some(StrokeDefinition::deserialize(child)),
                "locked" => circle.locked = true,
                "uuid" => circle.uuid = child.arguments.first().and_then(|a| Some(a.get_string())),
                _ => panic!("Unsupported child item type in FootprintCircle: {}", child.name),
            }
        }

        circle
    }
}

impl SyntaxItemSerializable for FootprintArc {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.start.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
        ];

        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(width, PositionPreference::None)));
        }

        if let Some(angle) = self.angle {
            children.push(SyntaxItem::from_single_argument("angle", SyntaxArgument::Number(angle, PositionPreference::None)));
        }

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        if let Some(mid) = &self.mid {
            children.insert(1, mid.serialize());
        }

        if let Some(stroke) = &self.stroke {
            children.push(stroke.serialize());
        }

        SyntaxItem {
            name: "fp_arc".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut arc = Self {
            layer: PcbLayer::FCu,
            start: Scalar2D::default(),
            mid: None,
            end: Scalar2D::default(),
            width: None,
            angle: None,
            stroke: syntax.get_named_child("stroke").and_then(|child| Some(StrokeDefinition::deserialize(child))),
            uuid: syntax.get_named_child("uuid").and_then(|child| Some(child.arguments.first().unwrap().get_string())),
            locked: syntax.get_named_child("locked").is_some(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "layer" => arc.layer = PcbLayer::from(&child),
                "start" => arc.start = Scalar2D::deserialize(child),
                "mid" => arc.mid = Some(Scalar2D::deserialize(child)),
                "end" => arc.end = Scalar2D::deserialize(child),
                "width" => arc.width = Some(child.arguments.get(0).unwrap().get_number()),
                "angle" => arc.angle = Some(child.arguments.get(0).unwrap().get_number()),
                "stroke" => arc.stroke = Some(StrokeDefinition::deserialize(child)),
                "locked" => arc.locked = true,
                "uuid" => arc.uuid = child.arguments.first().and_then(|a| Some(a.get_string())),
                _ => panic!("Unsupported child item type in FootprintArc: {}", child.name),
            }
        }

        arc
    }
}

impl SyntaxItemSerializable for FootprintRectangle {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.start.serialize(),
            self.end.serialize(),
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
        ];

        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument("width", SyntaxArgument::Number(width, PositionPreference::None)));
        }

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        if let Some(fill) = &self.fill {
            children.push(SyntaxItem::from_single_argument("fill", SyntaxArgument::Identifier((if *fill { "yes" } else { "no" }).into(), PositionPreference::None)));
        }

        if let Some(stroke) = &self.stroke {
            children.push(stroke.serialize());
        }

        SyntaxItem {
            name: "fp_rect".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut rectangle = Self {
            layer: PcbLayer::FCu,
            start: Scalar2D::default(),
            end: Scalar2D::default(),
            width: None,
            fill: None,
            stroke: syntax.get_named_child("stroke").and_then(|child| Some(StrokeDefinition::deserialize(child))),
            uuid: syntax.get_named_child("uuid").and_then(|child| Some(child.arguments.first().unwrap().get_string())),
            locked: syntax.get_named_child("locked").is_some(),
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "layer" => rectangle.layer = PcbLayer::from(&child),
                "start" => rectangle.start = Scalar2D::deserialize(child),
                "end" => rectangle.end = Scalar2D::deserialize(child),
                "width" => rectangle.width = Some(child.arguments.get(0).unwrap().get_number()),
                "stroke" => rectangle.stroke = Some(StrokeDefinition::deserialize(child)),
                "fill" => rectangle.fill = child.arguments.get(0).and_then(|s| Some(s.get_string() == "yes" || s.get_string() == "filled")),
                "locked" => rectangle.locked = true,
                "uuid" => rectangle.uuid = child.arguments.first().and_then(|a| Some(a.get_string())),
                _ => panic!("Unsupported child item type in FootprintRectangle: {}", child.name),
            }
        }

        rectangle
    }
}

impl SyntaxItemSerializable for FootprintText {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.position.serialize(),
            SyntaxItem::from_single_argument("layer", SyntaxArgument::Identifier(self.layer.to_string(), PositionPreference::None)),
            self.effects.serialize(),
        ];

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        let mut arguments = vec![
            SyntaxArgument::Identifier(match self.text_type {
                FootprintTextType::Reference => "reference".into(),
                FootprintTextType::Value => "value".into(),
                FootprintTextType::User => "user".into(),
            }, PositionPreference::None),
            SyntaxArgument::QuotedString(self.text.clone(), PositionPreference::None),
        ];
        if self.hide {
            arguments.push(SyntaxArgument::Identifier("hide".into(), PositionPreference::None));
        }
        if let Some(unlocked) = self.unlocked {
            children.push(SyntaxItem::from_single_argument("unlocked", SyntaxArgument::Identifier((match unlocked {
                true => "yes",
                false => "no",
            }).into(), PositionPreference::None)));
        }

        SyntaxItem {
            name: "fp_text".into(),
            arguments,
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut text = Self {
            text_type: match syntax.arguments.get(0).unwrap().get_string().as_str() {
                "reference" => FootprintTextType::Reference,
                "value" => FootprintTextType::Value,
                "user" => FootprintTextType::User,
                str => panic!("Unsupported footprint text type: {}", str),
            },
            text: syntax.arguments.get(1).unwrap().get_string(),
            position: Position::default(),
            unlocked: None,
            hide: syntax.has_argument(SyntaxArgument::Identifier("hide".to_string(), PositionPreference::None)),
            layer: PcbLayer::FCu,
            effects: TextEffect::default(),
            uuid: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "layer" => text.layer = PcbLayer::from(child),
                "effects" => text.effects = TextEffect::deserialize(child),
                "at" => text.position = Position::deserialize(child),
                "unlocked" => text.unlocked = Some(child.arguments.get(0).is_some_and(|a| a.get_string() == "yes")),
                "uuid" => text.uuid = child.arguments.first().and_then(|a| Some(a.get_string())),
                "hide" => text.hide = child.arguments.first().is_some_and(|a| a.get_string() == "yes"),
                "render_cache" => {} // life is complicated enough already, no need to make it even worse
                _ => panic!("Unsupported child item type in FootprintText: {}", child.name),
            }
        }

        text
    }
}

impl SyntaxItemSerializable for FootprintPad {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            self.position.serialize(),
            self.size.serialize(),
        ];

        if let Some(drill) = &self.drill {
            children.push(drill.serialize());
        }
        children.push(self.layers.serialize());

        if let Some(round_rect_ratio) = self.round_rect_ratio {
            children.push(SyntaxItem::from_single_argument("roundrect_rratio", SyntaxArgument::Number(round_rect_ratio, PositionPreference::None)));
        }
        if let Some(solder_mask_margin) = self.solder_mask_margin {
            children.push(SyntaxItem::from_single_argument("solder_mask_margin", SyntaxArgument::Number(solder_mask_margin, PositionPreference::None)));
        }
        if let Some(solder_paste_margin) = self.solder_paste_margin {
            children.push(SyntaxItem::from_single_argument("solder_paste_margin", SyntaxArgument::Number(solder_paste_margin, PositionPreference::None)));
        }
        if let Some(solder_paste_margin_ratio) = self.solder_paste_margin_ratio {
            children.push(SyntaxItem::from_single_argument("solder_paste_margin_ratio", SyntaxArgument::Number(solder_paste_margin_ratio, PositionPreference::None)));
        }
        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }
        if let Some(options) = &self.options {
            children.push(options.serialize());
        }
        if let Some(primitives) = &self.primitives {
            children.push(primitives.serialize());
        }
        if let Some(zone_connect) = &self.zone_connection {
            children.push(SyntaxItem::from_single_argument("zone_connect", SyntaxArgument::Number(match zone_connect {
                ZoneConnectMode::NotConnected => 0,
                ZoneConnectMode::ThermalRelief => 1,
                ZoneConnectMode::SolidFill => 2,
            } as f32, PositionPreference::None)));
        }
        if let Some(remove_unused_layer) = self.remove_unused_layer {
            children.push(SyntaxItem::from_single_argument("remove_unused_layers", SyntaxArgument::Identifier((match remove_unused_layer {
                true => "yes",
                false => "no",
            }).into(), PositionPreference::None)));
        }
        if let Some(property) = &self.property {
            children.push(SyntaxItem::from_single_argument("property", SyntaxArgument::Identifier((match property {
                PadProperty::Bga => "pad_prop_bga",
                PadProperty::FiducialGlob => "pad_prop_fiducial_glob",
                PadProperty::FiducialLoc => "pad_prop_fiducial_loc",
                PadProperty::TestPoint => "pad_prop_testpoint",
                PadProperty::HeatSink => "pad_prop_heatsink",
                PadProperty::Castellated => "pad_prop_castellated",
            }).into(), PositionPreference::None)));
        }

        let mut arguments = vec![
            match self.number.as_str() {
                "" => SyntaxArgument::QuotedString("".into(), PositionPreference::Start),
                str => SyntaxArgument::Identifier(str.into(), PositionPreference::Start),
            },
            SyntaxArgument::Identifier(match self.pad_type {
                PadType::ThruHole => "thru_hole",
                PadType::Smd => "smd",
                PadType::Connect => "connect",
                PadType::NpThruHole => "np_thru_hole",
            }.into(), PositionPreference::None),
            SyntaxArgument::Identifier(match self.pad_shape {
                PadShape::Circle => "circle",
                PadShape::Rect => "rect",
                PadShape::Oval => "oval",
                PadShape::Trapezoid => "trapezoid",
                PadShape::RoundRect => "roundrect",
                PadShape::Custom => "custom",
            }.into(), PositionPreference::None),
        ];

        SyntaxItem {
            name: "pad".into(),
            children,
            arguments,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut pad = Self {
            number: syntax.arguments.get(0).unwrap().get_string(),
            pad_type: match syntax.arguments.get(1).unwrap().get_string().as_str() {
                "thru_hole" => PadType::ThruHole,
                "smd" => PadType::Smd,
                "connect" => PadType::Connect,
                "np_thru_hole" => PadType::NpThruHole,
                str => panic!("Unsupported pad type: {}", str),
            },
            pad_shape: match syntax.arguments.get(2).unwrap().get_string().as_str() {
                "circle" => PadShape::Circle,
                "rect" => PadShape::Rect,
                "oval" => PadShape::Oval,
                "trapezoid" => PadShape::Trapezoid,
                "roundrect" => PadShape::RoundRect,
                "custom" => PadShape::Custom,
                str => panic!("Unsupported pad shape: {}", str),
            },
            position: Position::deserialize(syntax.get_named_child("at").unwrap()),
            size: Scalar2D::deserialize(syntax.get_named_child("size").unwrap()),
            drill: syntax.get_named_child("drill").map(|s| DrillDefinition::deserialize(s)),
            layers: syntax.get_named_child("layers").map(|s| Vec::<PcbLayer>::deserialize(s)).unwrap(),
            property: syntax.get_named_child("property").map(|s| match s.arguments.first().unwrap().get_string().as_str() {
                "pad_prop_bga" => PadProperty::Bga,
                "pad_prop_fiducial_glob" => PadProperty::FiducialGlob,
                "pad_prop_fiducial_loc" => PadProperty::FiducialLoc,
                "pad_prop_testpoint" => PadProperty::TestPoint,
                "pad_prop_heatsink" => PadProperty::HeatSink,
                "pad_prop_castellated" => PadProperty::Castellated,
                str => panic!("Unsupported pad property value: {}", str),
            }),
            remove_unused_layer: syntax.get_named_child("remove_unused_layer")
                .or_else(|| syntax.get_named_child("remove_unused_layers"))
                .map(|s| s.arguments.first().unwrap().get_string() == "yes"),
            keep_end_layers: None,
            round_rect_ratio: syntax.get_named_child("roundrect_rratio").map(|s| s.arguments.get(0).unwrap().get_number()),
            chamfer_ratio: None,
            chamfer: vec![],
            net: None,
            uuid: syntax.get_named_child("uuid").map(|s| s.arguments.get(0).unwrap().get_string()),
            pin_function: None,
            pin_type: None,
            die_length: None,
            zone_connection: syntax.get_named_child("zone_connection")
                .or_else(|| syntax.get_named_child("zone_connect"))
                .map(|s| match s.arguments.first().unwrap().get_number() as u8 {
                    0 => ZoneConnectMode::NotConnected,
                    1 => ZoneConnectMode::ThermalRelief,
                    2 => ZoneConnectMode::SolidFill,
                    num => panic!("Unsupported zone connect mode: {}", num),
                }),
            solder_mask_margin: syntax.get_named_child("solder_mask_margin").map(|s| s.arguments.get(0).unwrap().get_number()),
            solder_paste_margin: syntax.get_named_child("solder_paste_margin").map(|s| s.arguments.get(0).unwrap().get_number()),
            solder_paste_margin_ratio: syntax.get_named_child("solder_paste_margin_ratio").map(|s| s.arguments.get(0).unwrap().get_number()),
            clearance: None,
            locked: false,
            options: syntax.get_named_child("options").map(|s| FootprintPadOptions::deserialize(s)),
            primitives: syntax.get_named_child("primitives").map(|s| FootprintPadPrimitives::deserialize(s)),
        };

        pad
    }
}

impl SyntaxItemSerializable for FootprintZone {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("net", SyntaxArgument::Number(self.net as f32, PositionPreference::None)),
            SyntaxItem::from_single_argument("net_name", SyntaxArgument::QuotedString(self.net_name.clone(), PositionPreference::None)),
        ];

        let mut layers = self.layer.serialize();
        if self.layer.len() == 1 {
            layers.name = "layer".into();
        } else if self.layer.len() > 1 {
            layers.name = "layers".into();
        }
        children.push(layers);

        if let Some(uuid) = &self.uuid {
            children.push(SyntaxItem::from_single_argument("uuid", SyntaxArgument::QuotedString(uuid.clone(), PositionPreference::None)));
        }

        if let Some(name) = &self.name {
            children.push(SyntaxItem::from_single_argument("name", SyntaxArgument::QuotedString(name.clone(), PositionPreference::None)));
        }

        children.push(SyntaxItem::from_arguments("hatch", vec![
            SyntaxArgument::Identifier(match self.hatch_style {
                HatchStyle::None => "none",
                HatchStyle::Edge => "edge",
                HatchStyle::Full => "full",
            }.into(), PositionPreference::Start),
            SyntaxArgument::Number(self.hatch_pitch, PositionPreference::End),
        ]));

        if let Some(priority) = &self.priority {
            children.push(SyntaxItem::from_single_argument("name", SyntaxArgument::Number(*priority as f32, PositionPreference::None)));
        }

        children.push(self.connect_pads.serialize());
        children.push(SyntaxItem::from_single_argument("min_thickness", SyntaxArgument::Number(self.min_thickness, PositionPreference::None)));

        if let Some(flag) = &self.filled_areas_thickness {
            children.push(SyntaxItem::from_single_argument("filled_areas_thickness", SyntaxArgument::Identifier((if *flag { "yes" } else { "no" }).into(), PositionPreference::None)));
        }

        if let Some(settings) = &self.keepout_settings {
            children.push(settings.serialize());
        }

        children.push(self.fill_settings.serialize());

        let mut polygon = self.coordinate_points.serialize();
        polygon.name = "polygon".to_string();
        children.push(polygon);

        SyntaxItem {
            name: "zone".into(),
            arguments: vec![],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            net: syntax.get_named_child("net").unwrap().arguments.first().unwrap().get_number() as u32,
            net_name: syntax.get_named_child("net_name").unwrap().arguments.first().unwrap().get_string(),
            layer: Vec::<PcbLayer>::deserialize(syntax.get_named_child("layer").unwrap_or_else(|| syntax.get_named_child("layers").unwrap())),
            uuid: syntax.get_named_child("uuid").map(|s| s.arguments.first().unwrap().get_string()),
            name: syntax.get_named_child("name").map(|s| s.arguments.first().unwrap().get_string()),
            hatch_style: syntax.get_named_child("hatch").unwrap().arguments.first().map(|a| match a.get_string().as_str() {
                "none" => HatchStyle::None,
                "edge" => HatchStyle::Edge,
                "full" => HatchStyle::Full,
                str => panic!("Invalid footprint zone hatch style: {}", str),
            }).unwrap(),
            hatch_pitch: syntax.get_named_child("hatch").unwrap().arguments.last().unwrap().get_number(),
            priority: syntax.get_named_child("priority").map(|s| s.arguments.first().unwrap().get_number() as u32),
            connect_pads: syntax.get_named_child("connect_pads").map(|p| FootprintZoneConnectPads::deserialize(p)).unwrap(),
            min_thickness: syntax.get_named_child("min_thickness").unwrap().arguments.first().unwrap().get_number(),
            filled_areas_thickness: syntax.get_named_child("filled_areas_thickness").map(|s| s.arguments
                .first().unwrap().get_string() != "no"),
            keepout_settings: syntax.get_named_child("keepout").map(|p| FootprintZoneKeepoutSettings::deserialize(p)),
            fill_settings: syntax.get_named_child("fill").map(|p| FootprintZoneFillSettings::deserialize(p)).unwrap(),
            coordinate_points: syntax.get_named_child("polygon").map(|p| GraphicPolygon::deserialize(p)).unwrap(),
        }
    }
}

impl SyntaxItemSerializable for FootprintZoneConnectPads {
    fn serialize(&self) -> SyntaxItem {
        let mut item = SyntaxItem {
            name: "connect_pads".into(),
            children: vec![
                SyntaxItem::from_single_argument("clearance", SyntaxArgument::Number(self.clearance, PositionPreference::None))
            ],
            arguments: vec![],
        };

        if let Some(connection_type) = &self.connection_type {
            item.arguments.push(SyntaxArgument::Identifier(match connection_type {
                PadConnectionType::ThruHoleOnly => "thru_hole_only",
                PadConnectionType::Full => "full",
                PadConnectionType::No => "no",
            }.to_string(), PositionPreference::None));
        }

        item
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut connect_pads = Self {
            connection_type: None,
            clearance: syntax.get_named_child("clearance").unwrap().arguments.first().unwrap().get_number(),
        };

        if let Some(connection_type) = syntax.arguments.first() {
            connect_pads.connection_type = Some(match connection_type.get_string().as_str() {
                "thru_hole_only" => PadConnectionType::ThruHoleOnly,
                "full" => PadConnectionType::Full,
                "no" => PadConnectionType::No,
                str => panic!("Invalid connection pad type: {}", str),
            });
        }

        connect_pads
    }
}

impl SyntaxItemSerializable for FootprintZoneKeepoutSettings {
    fn serialize(&self) -> SyntaxItem {
        let children = vec![
            SyntaxItem::from_single_argument(
                "tracks",
                SyntaxArgument::Identifier(
                    if self.tracks_allowed { "allowed" } else { "not_allowed" }.into(),
                    PositionPreference::None
                )
            ),
            SyntaxItem::from_single_argument(
                "vias",
                SyntaxArgument::Identifier(
                    if self.vias_allowed { "allowed" } else { "not_allowed" }.into(),
                    PositionPreference::None
                )
            ),
            SyntaxItem::from_single_argument(
                "pads",
                SyntaxArgument::Identifier(
                    if self.pads_allowed { "allowed" } else { "not_allowed" }.into(),
                    PositionPreference::None
                )
            ),
            SyntaxItem::from_single_argument(
                "copperpour",
                SyntaxArgument::Identifier(
                    if self.copper_pour_allowed { "allowed" } else { "not_allowed" }.into(),
                    PositionPreference::None
                )
            ),
            SyntaxItem::from_single_argument(
                "footprints",
                SyntaxArgument::Identifier(
                    if self.footprints_allowed { "allowed" } else { "not_allowed" }.into(),
                    PositionPreference::None
                )
            ),
        ];

        SyntaxItem {
            name: "keepout".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut settings = Self {
            tracks_allowed: false,
            vias_allowed: false,
            pads_allowed: false,
            copper_pour_allowed: false,
            footprints_allowed: false,
        };

        for child in &syntax.children {
            let allowed = child.arguments.first().unwrap().get_string() == "allowed";
            match child.name.as_str() {
                "tracks" => settings.tracks_allowed = allowed,
                "vias" => settings.vias_allowed = allowed,
                "pads" => settings.pads_allowed = allowed,
                "copperpour" => settings.copper_pour_allowed = allowed,
                "footprints" => settings.footprints_allowed = allowed,
                _ => panic!("Unsupported child item type in FootprintZoneKeepoutSettings: {}", child.name),
            }
        }

        settings
    }
}

impl SyntaxItemSerializable for FootprintZoneFillSettings {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![];

        if let ZoneFillMode::Hatched = self.mode {
            children.push(SyntaxItem::from_single_argument("mode", SyntaxArgument::Identifier("hatched".into(), PositionPreference::None)));
        }

        children.push(SyntaxItem::from_single_argument("thermal_gap", SyntaxArgument::Number(self.thermal_gap, PositionPreference::None)));
        children.push(SyntaxItem::from_single_argument("thermal_bridge_width", SyntaxArgument::Number(self.thermal_bridge_width, PositionPreference::None)));

        if let Some(smoothing) = &self.smoothing {
            children.push(SyntaxItem::from_single_argument("smoothing", SyntaxArgument::Identifier((match smoothing {
                ZoneSmoothingStyle::Chamfer => "chamfer",
                ZoneSmoothingStyle::Fillet => "fillet",
            }).into(), PositionPreference::None)));
        }

        if let Some(radius) = self.radius {
            children.push(SyntaxItem::from_single_argument("radius", SyntaxArgument::Number(radius, PositionPreference::None)));
        }

        if let Some(mode) = &self.island_removal_mode {
            children.push(SyntaxItem::from_single_argument("island_removal_mode", SyntaxArgument::Identifier(match mode {
                ZoneIslandRemovalMode::AlwaysRemove => "0",
                ZoneIslandRemovalMode::NeverRemove => "1",
                ZoneIslandRemovalMode::MinimumArea => "2",
            }.into(), PositionPreference::None)))
        }

        if let Some(area) = self.island_area_min {
            children.push(SyntaxItem::from_single_argument("island_area_min", SyntaxArgument::Number(area, PositionPreference::None)));
        }

        if let Some(thickness) = self.hatch_thickness {
            children.push(SyntaxItem::from_single_argument("hatch_thickness", SyntaxArgument::Number(thickness, PositionPreference::None)));
        }

        if let Some(gap) = self.hatch_gap {
            children.push(SyntaxItem::from_single_argument("hatch_gap", SyntaxArgument::Number(gap, PositionPreference::None)));
        }

        if let Some(orientation) = self.hatch_orientation {
            children.push(SyntaxItem::from_single_argument("hatch_orientation", SyntaxArgument::Number(orientation, PositionPreference::None)));
        }

        if let Some(value) = self.hatch_smoothing_value {
            children.push(SyntaxItem::from_single_argument("hatch_smoothing_value", SyntaxArgument::Number(value, PositionPreference::None)));
        }

        if let Some(level) = &self.hatch_smoothing_level {
            children.push(SyntaxItem::from_single_argument("island_removal_mode", SyntaxArgument::Identifier(match level {
                HatchSmoothingLevel::NoSmoothing => "0",
                HatchSmoothingLevel::Fillet => "1",
                HatchSmoothingLevel::ArcMinimum => "2",
                HatchSmoothingLevel::ArcMaximum => "3",
            }.into(), PositionPreference::None)))
        }

        if let Some(algo) = &self.hatch_border_algorithm {
            children.push(SyntaxItem::from_single_argument("hatch_border_algorithm", SyntaxArgument::Identifier(match algo {
                HatchBorderAlgorithm::ZoneMinimumThickness => "0",
                HatchBorderAlgorithm::HatchThickness => "1",
            }.into(), PositionPreference::None)))
        }

        if let Some(area) = self.hatch_min_hole_area {
            children.push(SyntaxItem::from_single_argument("hatch_min_hole_area", SyntaxArgument::Number(area, PositionPreference::None)));
        }

        SyntaxItem {
            name: "fill".into(),
            arguments: self.fill.map(|f| vec![SyntaxArgument::Identifier((if f { "yes" } else { "no" }).into(), PositionPreference::None)]).unwrap_or(vec![]),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut fill = Self {
            fill: None,
            mode: ZoneFillMode::Solid,
            thermal_gap: 0.0,
            thermal_bridge_width: 0.0,
            smoothing: None,
            radius: None,
            island_removal_mode: None,
            island_area_min: None,
            hatch_thickness: None,
            hatch_gap: None,
            hatch_orientation: None,
            hatch_smoothing_level: None,
            hatch_smoothing_value: None,
            hatch_border_algorithm: None,
            hatch_min_hole_area: None,
        };

        if let Some(arg) = syntax.arguments.first() {
            fill.fill = Some(arg.get_string() == "yes");
        }

        for child in &syntax.children {
            let first_argument = child.arguments.first().unwrap();
            match child.name.as_str() {
                "mode" => fill.mode = match first_argument.get_string().as_str() {
                    "hatched" => ZoneFillMode::Hatched,
                    str => panic!("Invalid mode argument in FootprintZoneFillSettings: {}", str),
                },
                "thermal_gap" => fill.thermal_gap = first_argument.get_number(),
                "thermal_bridge_width" => fill.thermal_bridge_width = first_argument.get_number(),
                "smoothing" => fill.smoothing = Some(match first_argument.get_string().as_str() {
                    "chamfer" => ZoneSmoothingStyle::Chamfer,
                    "fillet" => ZoneSmoothingStyle::Fillet,
                    str => panic!("Invalid smoothing argument in FootprintZoneFillSettings: {}", str),
                }),
                "radius" => fill.radius = Some(first_argument.get_number()),
                "island_removal_mode" => fill.island_removal_mode = Some(match first_argument.get_string().as_str() {
                    "0" => ZoneIslandRemovalMode::AlwaysRemove,
                    "1" => ZoneIslandRemovalMode::NeverRemove,
                    "2" => ZoneIslandRemovalMode::MinimumArea,
                    str => panic!("Invalid island removal mode argument in FootprintZoneFillSettings: {}", str),
                }),
                "island_area_min" => fill.island_area_min = Some(first_argument.get_number()),
                "hatch_thickness" => fill.hatch_thickness = Some(first_argument.get_number()),
                "hatch_gap" => fill.hatch_gap = Some(first_argument.get_number()),
                "hatch_orientation" => fill.hatch_orientation = Some(first_argument.get_number()),
                "hatch_smoothing_level" => fill.hatch_smoothing_level = Some(match first_argument.get_string().as_str() {
                    "0" => HatchSmoothingLevel::NoSmoothing,
                    "1" => HatchSmoothingLevel::Fillet,
                    "2" => HatchSmoothingLevel::ArcMinimum,
                    "3" => HatchSmoothingLevel::ArcMaximum,
                    str => panic!("Invalid smoothing argument in FootprintZoneFillSettings: {}", str),
                }),
                "hatch_smoothing_value" => fill.hatch_smoothing_value = Some(first_argument.get_number()),
                "hatch_border_algorithm" => fill.hatch_border_algorithm = Some(match first_argument.get_string().as_str() {
                    "0" => HatchBorderAlgorithm::ZoneMinimumThickness,
                    "1" => HatchBorderAlgorithm::HatchThickness,
                    str => panic!("Invalid hatch border algorithm argument in FootprintZoneFillSettings: {}", str),
                }),
                "hatch_min_hole_area" => fill.hatch_min_hole_area = Some(first_argument.get_number()),
                str => panic!("Unsupported child item type in FootprintZoneFillSettings: {}", str),
            }
        }

        fill
    }
}

impl SyntaxItemSerializable for FootprintPadOptions {
    fn serialize(&self) -> SyntaxItem {
        SyntaxItem {
            name: "options".into(),
            arguments: vec![],
            children: vec![
                SyntaxItem::from_single_argument("clearance", SyntaxArgument::Identifier(match self.clearance {
                    ClearanceType::Outline => "outline",
                    ClearanceType::ConvexHull => "convexhull",
                }.into(), PositionPreference::None)),
                SyntaxItem::from_single_argument("anchor", SyntaxArgument::Identifier(match self.anchor {
                    AnchorType::Rect => "rect",
                    AnchorType::Circle => "circle",
                }.into(), PositionPreference::None)),
            ],
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            clearance: match syntax.get_named_child("clearance").unwrap().arguments.first().unwrap().get_string().as_str() {
                "outline" => ClearanceType::Outline,
                "convexhull" => ClearanceType::ConvexHull,
                str => panic!("Invalid pad clearance type: {}", str),
            },
            anchor: match syntax.get_named_child("anchor").unwrap().arguments.first().unwrap().get_string().as_str() {
                "rect" => AnchorType::Rect,
                "circle" => AnchorType::Circle,
                str => panic!("Invalid pad anchor type: {}", str),
            },
        }
    }
}

impl SyntaxItemSerializable for FootprintPadPrimitives {
    fn serialize(&self) -> SyntaxItem {
        let mut children = Vec::new();

        // Add optional width if present
        if let Some(width) = self.width {
            children.push(SyntaxItem::from_single_argument(
                "width",
                SyntaxArgument::Number(width, PositionPreference::None),
            ));
        }

        // Add optional fill if present
        if let Some(fill) = self.fill {
            children.push(SyntaxItem::from_single_argument(
                "fill",
                SyntaxArgument::Identifier(
                    if fill { "solid" } else { "none" }.into(),
                    PositionPreference::None,
                ),
            ));
        }

        // Add all graphic primitives
        for line in &self.lines {
            children.push(line.serialize());
        }
        for rect in &self.rectangles {
            children.push(rect.serialize());
        }
        for arc in &self.arcs {
            children.push(arc.serialize());
        }
        for circle in &self.circles {
            children.push(circle.serialize());
        }
        for curve in &self.curves {
            children.push(curve.serialize());
        }
        for polygon in &self.polygons {
            children.push(polygon.serialize());
        }
        for box_annotation in &self.annotation_boxes {
            children.push(box_annotation.serialize());
        }

        SyntaxItem {
            name: "primitives".into(),
            arguments: Vec::new(),
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut primitives = Self {
            lines: Vec::new(),
            rectangles: Vec::new(),
            arcs: Vec::new(),
            circles: Vec::new(),
            curves: Vec::new(),
            polygons: Vec::new(),
            annotation_boxes: Vec::new(),
            width: None,
            fill: None,
        };

        for child in &syntax.children {
            match child.name.as_str() {
                "width" => primitives.width = Some(child.arguments.get(0).unwrap().get_number()),
                "fill" => {
                    let fill_type = child.arguments.first().unwrap().get_string();
                    primitives.fill = Some(fill_type == "solid" || fill_type == "yes");
                }
                "gr_line" => primitives.lines.push(GraphicLine::deserialize(child)),
                "gr_rect" => primitives.rectangles.push(GraphicRectangle::deserialize(child)),
                "gr_arc" => primitives.arcs.push(GraphicArc::deserialize(child)),
                "gr_circle" => primitives.circles.push(GraphicCircle::deserialize(child)),
                "bezier" => primitives.curves.push(GraphicCurve::deserialize(child)),
                "gr_poly" => primitives.polygons.push(GraphicPolygon::deserialize(child)),
                "gr_bbox" => primitives.annotation_boxes.push(GraphicAnnotationBox::deserialize(child)),
                _ => panic!("Unsupported child item type in FootprintPadPrimitives: {}", child.name),
            }
        }

        primitives
    }
}

impl SyntaxItemSerializable for Vec<PcbLayer> {
    fn serialize(&self) -> SyntaxItem {
        let mut arguments = vec![];
        let mut list = self.clone();

        let layers_by_name = PcbLayer::iter()
            .chunk_by(|l| {
                let name = l.to_string();
                let parts = name.split('.');
                let parts = parts.collect::<Vec<&str>>();
                parts[1].to_string()
            })
            .into_iter()
            .map(|(a, b)| (a, b.collect::<Vec<_>>()))
            .collect::<Vec<_>>();

        for (layer, names) in layers_by_name.into_iter() {
            if !names.iter().all(|l| list.contains(l)) {
                continue;
            }

            list = list.into_iter().filter(|l| !names.contains(l)).collect();
            arguments.push(SyntaxArgument::Identifier(format!("*.{}", layer), PositionPreference::None));
        }

        for layer in list {
            arguments.push(SyntaxArgument::Identifier(layer.to_string(), PositionPreference::None));
        }

        SyntaxItem {
            name: "layers".into(),
            children: vec![],
            arguments,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let layer_map = PcbLayer::iter()
            .map(|layer| (layer.to_string(), layer))
            .collect::<HashMap<_, _>>();

        let mut layers = Vec::<PcbLayer>::new();

        for argument in &syntax.arguments {
            let argument = argument.get_string();

            let parts = argument.split('.').collect::<Vec<&str>>();
            layers.extend(layer_map.iter().filter(|(a, b)| {
                let this_parts = a.split('.').collect::<Vec<&str>>();
                let type_match = this_parts[0] == parts[0] || parts[0] == "*";
                let name_match = this_parts[1] == parts[1] || parts[1] == "*";

                if name_match && parts[0].contains("&") {
                    let type_list = parts[0].split("&").collect::<Vec<&str>>();
                    if type_list.contains(&this_parts[0].as_ref()) {
                        return true;
                    }
                }

                type_match && name_match
            }).map(|(_, b)| *b));
        }

        layers
    }
}

impl SyntaxItemSerializable for DrillDefinition {
    fn serialize(&self) -> SyntaxItem {
        let mut arguments = vec![];
        if self.oval {
            arguments.push(SyntaxArgument::Identifier("oval".into(), PositionPreference::Start));
        }
        arguments.push(SyntaxArgument::Number(self.diameter, PositionPreference::None));
        if let Some(width) = self.width {
            arguments.push(SyntaxArgument::Number(width, PositionPreference::None));
        }

        let mut children = vec![];
        if let Some(offset) = &self.offset {
            children.push(offset.serialize());
        }

        SyntaxItem {
            name: "drill".into(),
            children,
            arguments,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        let mut definition = Self {
            oval: false,
            width: None,
            diameter: 0.0,
            offset: None,
        };

        let mut num_index = 0;
        for argument in &syntax.arguments {
            match argument {
                SyntaxArgument::Number(num, _) => {
                    if num_index == 0 {
                        definition.diameter = *num;
                    } else if num_index == 1 {
                        definition.width = Some(*num);
                    }
                    num_index += 1;
                }
                SyntaxArgument::Identifier(identifier, _) => {
                    if identifier == "oval" {
                        definition.oval = true;
                    }
                }
                _ => {}
            }
        }

        if let Some(offset) = syntax.get_named_child("offset") {
            definition.offset = Some(Scalar2D::deserialize(&offset));
        }

        definition
    }
}

impl TopLevelSerializable for FootprintLibrary {
    fn get_same_line_identifiers() -> Vec<String> {
        Vec::from([
            "layer", "layers", "xyz", "thickness", "start", "mid", "end", "width", "angle",
            "font", "size", "thickness", "at", "drill", "offset", "solder_mask_margin",
            "roundrect_rratio", "net", "net_name", "hatch", "clearance", "thermal_gap",
            "thermal_bridge_width", "tracks", "vias", "pads", "copperpour", "footprints"
        ]).iter().map(|s| s.to_string()).collect()
    }
}