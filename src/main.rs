use crate::args::{CliArguments, Command};
use crate::easyeda::api::component_data::ComponentDataResponse;
use crate::easyeda::api::product_data::ProductDataResponse;
use crate::easyeda::footprint::EasyEDAFootprint;
use crate::easyeda::symbol::EasyEDASymbol;
use crate::kicad::model::footprint_lib_table::{FootprintLibTable, FootprintLibTableItem};
use crate::kicad::model::footprint_library::{FootprintLibrary, FootprintModel, Scalar3D};
use crate::kicad::model::symbol_lib_table::{SymbolLibTable, SymbolLibTableItem};
use crate::kicad::model::symbol_library::{Symbol, SymbolLib};
use crate::kicad::syntax::{KiCadParser, SyntaxItemSerializable};
use clap::Parser;
use itertools::Itertools;
use opencascade::primitives::Shape;
use std::fs;

mod kicad;
mod easyeda;
mod dev;
mod args;

fn main() -> anyhow::Result<()> {
    let cli = CliArguments::parse();
    match cli.command {
        Command::Import { code, update, name, description, root } => {
            let project_root_dir = std::env::current_dir()?;

            let mut library_root_dir = std::env::current_dir()?;
            let library_name = sanitize_filename::sanitize(&name);
            let library_name = library_name.as_str();
            if let Some(root) = root {
                library_root_dir = library_root_dir.join(root);
                if !library_root_dir.exists() {
                    fs::create_dir_all(&library_root_dir)?;
                }
            }

            let library_path_relative = library_root_dir.to_str().unwrap().replace(project_root_dir.to_str().unwrap(), "${KIPRJMOD}");

            let lcsc_code = code[1..].parse::<u32>();
            if !code.starts_with("C") || lcsc_code.is_err() {
                return Err(anyhow::anyhow!("The provided LCSC code is in an invalid format."));
            }
            let lcsc_code = format!("C{}", lcsc_code?).clone();
            let lcsc_code = lcsc_code.as_str();

            println!("Importing '{}'...", lcsc_code);

            // Download component data
            let response = ureq::get(
                format!("https://pro.easyeda.com/api/eda/product/search?keyword={code}&currPage=1&pageSize=1")
            ).call()?;
            let body_string = response.into_body().read_to_string()?;
            let response = serde_json::from_str::<ProductDataResponse>(&body_string)?;
            let result = response.result.product_list.iter().find(|p| p.number == code);
            if let None = result {
                return Err(anyhow::anyhow!("Product code not found: '{}'", lcsc_code));
            }
            let component_result = result.unwrap();
            let device_name = component_result.mpn.clone();
            let safe_part_name = sanitize_filename::sanitize(&device_name);

            let mut symbol = EasyEDASymbol::parse(&component_result.device_info.symbol_info.data_str)?;
            let mut footprint = EasyEDAFootprint::parse(&component_result.device_info.footprint_info.data_str)?;

            symbol.part_number = Some(lcsc_code.into());
            footprint.part_number = Some(lcsc_code.into());

            let designator = symbol.get_designator().clone();

            let mut kicad_symbol: Symbol = symbol.try_into()?;
            let mut kicad_footprint: FootprintLibrary = footprint.try_into()?;

            kicad_symbol.symbol_id = device_name.clone();
            kicad_footprint.footprint_id = device_name.clone();

            // Add component properties
            kicad_symbol.add_hidden_property("Part Number", device_name.as_str());
            kicad_symbol.add_hidden_property("LCSC", lcsc_code);
            kicad_symbol.add_hidden_property("Footprint", format!("{library_name}:{device_name}").as_str());
            kicad_footprint.add_hidden_property("LCSC", lcsc_code);

            if let Some(datasheet) = component_result.device_info.attributes.get("Datasheet") {
                kicad_symbol.add_hidden_property("Datasheet", datasheet);
                kicad_footprint.add_hidden_property("Datasheet", datasheet);
            }
            if let Some(description) = component_result.device_info.attributes.get("Description").cloned().or_else(|| Some(component_result.device_info.description.clone())) {
                kicad_symbol.add_hidden_property("Description", &description);
                kicad_footprint.add_hidden_property("Description", &description);
                kicad_footprint.description = Some(description.clone());
            }
            if let Some(jlc_part_class) = component_result.device_info.attributes.get("JLCPCB Part Class") {
                kicad_symbol.add_hidden_property("JLCPCB Part Class", jlc_part_class);
                kicad_footprint.add_hidden_property("JLCPCB Part Class", jlc_part_class);
            }
            if let Some(value) = component_result.device_info.attributes.get("Value") {
                kicad_symbol.add_property("Value", value.as_str(), 0.0, 0.0);
            } else {
                kicad_symbol.add_property("Value", device_name.as_str(), 0.0, 0.0);
            }
            if let Some(designator) = designator {
                kicad_symbol.add_property("Reference", &designator, 0.0, 0.0);
            }

            // Check if symbol lib exists, create if it doesn't
            let symbol_lib_path = library_root_dir.join(format!("{library_name}.kicad_sym").as_str());
            let mut symbol_lib = match fs::exists(&symbol_lib_path)? {
                true => {
                    let lib_data = fs::read_to_string(&symbol_lib_path)?;
                    let tokens = KiCadParser::tokenize(&lib_data);
                    let item = KiCadParser::parse_syntax_item(&tokens);
                    let model: SymbolLib = SyntaxItemSerializable::deserialize(&item);
                    model
                }
                false => {
                    SymbolLib {
                        version: 20211014,
                        generator: "jlcrs".into(),
                        generator_version: None,
                        symbols: vec![],
                    }
                }
            };
            let existing_component = symbol_lib.symbols.iter_mut().find(|s| s.symbol_id == kicad_symbol.symbol_id);
            if !update && existing_component.is_some() {
                return Err(anyhow::anyhow!("This component has already been imported into the project, aborting. Use the --update flag to overwrite an existing component."));
            }
            if existing_component.is_none() {
                println!("Adding device '{}'...", device_name);
                symbol_lib.symbols.push(kicad_symbol);
            } else if let Some(existing_symbol) = existing_component {
                *existing_symbol = kicad_symbol;
            }

            // Download STEP model data
            if let Some(model_3d) = &component_result.device_info.footprint_info.model_3d {
                let model_id = &model_3d.uri;

                let response = ureq::get(format!("https://pro.easyeda.com/api/v2/components/{model_id}")).call();
                if let Ok(model_response) = response {
                    let body_string = model_response.into_body().read_to_string()?;
                    let component_data = serde_json::from_str::<ComponentDataResponse>(&body_string)?;
                    if let Some(product_result) = component_data.result {
                        let model_id = product_result.n3d_model_uuid;
                        let response = ureq::get(format!("https://modules.easyeda.com/qAxj6KHrDKw4blvCG8QJPs7Y/{model_id}")).call();
                        if let Ok(model_response) = response {
                            let body_string = model_response.into_body().read_to_string()?;
                            println!("Found STEP model, downloading...");
                            let model_directory = library_root_dir
                                .join(format!("{library_name}.pretty").as_str())
                                .join("models");
                            if !model_directory.exists() {
                                fs::create_dir_all(&model_directory)?;
                            }
                            let model_path = model_directory.join(format!("{safe_part_name}.step"));
                            fs::write(&model_path, body_string)?;

                            let shape = Shape::read_step(&model_path)?;
                            let bounding_box = shape.bounding_box();

                            let center_x = (bounding_box.max_x + bounding_box.min_x) / 2.0;
                            let center_y = (bounding_box.max_y + bounding_box.min_y) / 2.0;
                            let min_z = bounding_box.min_z;

                            let model_transform = model_3d.transform
                                .split(',')
                                .map(|f| f.parse::<f32>().unwrap())
                                .collect_vec();
                            let transform_offset = &model_transform[6..9].iter().map(|v| v * 0.0254).collect_vec();
                            let rotation = &model_transform[3..6].iter().rev().collect_vec();

                            //println!("origin: [{}, {}, {}]", center_x, center_y, min_z);
                            //println!("rotation: {:?}", &rotation);
                            //println!("offset: {:?}", &transform_offset);

                            let rotation_z = (*rotation[2]).to_radians();
                            let mul_y = rotation_z.cos();

                            let offset_x = -mul_y * center_x * 0.0393701 + transform_offset[0] * 0.0393701;
                            let offset_y = -mul_y * center_y * 0.0393701 + transform_offset[1] * 0.0393701;
                            let offset_z = -min_z * 0.0393701 + transform_offset[2] * 0.0393701;

                            kicad_footprint.model = Some(FootprintModel {
                                model_file: model_path.to_str().unwrap().replace(project_root_dir.to_str().unwrap(), "${KIPRJMOD}"),
                                opacity: None,
                                at: Some(Scalar3D::new("xyz", offset_x, offset_y, offset_z)),
                                rotate: Some(Scalar3D::new("xyz", -*rotation[0], -*rotation[1], -*rotation[2])),
                                scale: None,
                                offset: None,
                            });
                        }
                    } else {
                        println!("No STEP model was found for this component");
                    }
                } else {
                    println!("No STEP model was found for this component");
                }
            }

            let item_ser = symbol_lib.serialize();
            let tokens = KiCadParser::generate_tokens(&item_ser);
            let symbol_lib_data = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
            fs::write(symbol_lib_path, symbol_lib_data)?;

            // Save footprint to .pretty directory
            let footprint_lib_root = library_root_dir.join(format!("{library_name}.pretty").as_str());
            if !fs::exists(&footprint_lib_root)? {
                fs::create_dir(&footprint_lib_root)?;
            }
            let footprint_path = footprint_lib_root.join(format!("{safe_part_name}.kicad_mod").as_str());
            let item = kicad_footprint.serialize();
            let tokens = KiCadParser::generate_tokens(&item);
            let footprint_data = KiCadParser::stringify_tokens::<FootprintLibrary>(&tokens);
            fs::write(footprint_path, footprint_data)?;

            // Check if the sym-lib-table/fp-lib-table files exist, create them if they don't
            let sym_lib_table_path = project_root_dir.join("sym-lib-table");
            let mut sym_lib_table = match fs::exists(&sym_lib_table_path)? {
                true => {
                    let sym_lib_table_data = fs::read_to_string(&sym_lib_table_path.to_str().unwrap())?;
                    let tokens = KiCadParser::tokenize(&sym_lib_table_data);
                    let item = KiCadParser::parse_syntax_item(&tokens);
                    let model: SymbolLibTable = SyntaxItemSerializable::deserialize(&item);
                    model
                }
                false => {
                    SymbolLibTable {
                        version: 7,
                        libraries: vec![],
                    }
                }
            };
            if !sym_lib_table.libraries.iter().any(|e| e.name == library_name) {
                sym_lib_table.libraries.push(SymbolLibTableItem {
                    name: library_name.into(),
                    description,
                    hidden: false,
                    disabled: false,
                    lib_type: "KiCad".into(),
                    options: String::new(),
                    uri: format!("{library_path_relative}/{library_name}.kicad_sym").into(),
                });
                let items_ser = sym_lib_table.serialize();
                let tokens = KiCadParser::generate_tokens(&items_ser);
                let sym_lib_table_data = KiCadParser::stringify_tokens::<SymbolLibTable>(&tokens);
                fs::write(sym_lib_table_path, sym_lib_table_data)?;
            }

            let fp_lib_table_path = project_root_dir.join("fp-lib-table");
            let mut fp_lib_table = match fs::exists(&fp_lib_table_path)? {
                true => {
                    let fp_lib_table_data = fs::read_to_string(&fp_lib_table_path.to_str().unwrap())?;
                    let tokens = KiCadParser::tokenize(&fp_lib_table_data);
                    let item = KiCadParser::parse_syntax_item(&tokens);
                    let model: FootprintLibTable = SyntaxItemSerializable::deserialize(&item);
                    model
                }
                false => {
                    FootprintLibTable {
                        version: 7,
                        libraries: vec![],
                    }
                }
            };
            if !fp_lib_table.libraries.iter().any(|e| e.name == library_name) {
                fp_lib_table.libraries.push(FootprintLibTableItem {
                    name: library_name.into(),
                    description: "Components downloaded and converted directly from JLCPCB".into(),
                    disabled: false,
                    lib_type: "KiCad".into(),
                    options: String::new(),
                    uri: format!("{library_path_relative}/{library_name}.pretty").into(),
                });
                let items_ser = fp_lib_table.serialize();
                let tokens = KiCadParser::generate_tokens(&items_ser);
                let fp_lib_table_data = KiCadParser::stringify_tokens::<FootprintLibTable>(&tokens);
                fs::write(fp_lib_table_path, fp_lib_table_data)?;
            }

            println!("The component has been imported.");
        }
    }
    Ok(())
}

#[allow(unused)]
struct BoundingBox {
    min_x: f32,
    min_y: f32,
    min_z: f32,

    max_x: f32,
    max_y: f32,
    max_z: f32,
}

trait HasBoundingBox {
    fn bounding_box(&self) -> BoundingBox;
}

impl HasBoundingBox for Shape {
    fn bounding_box(&self) -> BoundingBox {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for vert in self.mesh().vertices {
            min_x = min_x.min(vert.x as f32);
            min_y = min_y.min(vert.y as f32);
            min_z = min_z.min(vert.z as f32);
            max_x = max_x.max(vert.x as f32);
            max_y = max_y.max(vert.y as f32);
            max_z = max_z.max(vert.z as f32);
        }

        BoundingBox {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }
}