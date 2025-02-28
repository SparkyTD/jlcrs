use std::fs;
use std::process::Command;
use std::time::Instant;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use regex::Regex;
use serde_json::{json, Value};
use crate::easyeda;
use crate::easyeda::footprint::EasyEDAFootprint;
use crate::easyeda::symbol::{EasyEDASymbol, SymbolElement};
use crate::kicad::model::footprint_library::FootprintLibrary;
use crate::kicad::model::symbol_library::SymbolLib;
use crate::kicad::syntax::{KiCadParser, SyntaxItemSerializable, TopLevelSerializable};

#[allow(unused)]
fn test_parse_file<T>(str: &str) -> anyhow::Result<bool>
where
    T: TopLevelSerializable,
{
    let tokens = KiCadParser::tokenize(&str);
    let item = KiCadParser::parse_syntax_item(&tokens);
    let model: T = SyntaxItemSerializable::deserialize(&item);

    let item_ser = model.serialize();
    let tokens = KiCadParser::generate_tokens(&item_ser);
    let string = KiCadParser::stringify_tokens::<T>(&tokens);

    let matches = item_ser.deep_equals(&item);

    println!("Match: {}", matches);

    if !matches {
        println!("{}", string);
    }

    Ok(matches)
}

#[allow(unused)]
fn dev_kicad_parser_match_test() -> anyhow::Result<()> {
    let test_input_sym = fs::read_to_string("/home/sparky/HardwareProjects/iot-controller/lfxp2-5e-5tn144.kicad_sym")?;

    test_parse_file::<SymbolLib>(&test_input_sym)?;

    Ok(())
}

#[allow(unused)]
fn dev_convert_component(lcsc_id: String) -> anyhow::Result<()> {
    // [OK] C1338621 - STM32-L1
    // [OK] C2765186 - USB-C
    // [OK] C136421 - HDMI
    // [OK] C105419 - MicroSD
    // [OK] C2803250 - SDRAM
    // [OK] C5446699 - RGB LED
    // [OK] C21190 - Resistor

    // [OK] C2913204 - ESP32-S3-WROOM-1-N8R2
    // [OK] C165948 - UsbC Connector - TYPE-C-31-M-12
    // [OK] C18191948 - MOSFET - IRLR7843TRPBF-JSM
    // [OK] C93168 - Relay - G6K-2F-Y DC3
    // [OK] C8545 - MOSFET - 2N7002
    // [OK] C474923 - Screw Terminal Block - KF128-2.54-5P
    // [OK] C474920 - Screw Terminal Block - KF128-2.54-2P
    // [OK] C318884 - SMD Button - TS-1187A-B-A-B
    // [OK] C2874116 - RGB LED - NH-B2020RGBA-HF
    // [OK] C841386 - DC-Dc Regulator TPS56637RPA
    let (mut symbol, footprint) = easyeda::tests::download_component(lcsc_id.as_str())?;

    let is_complex_symbol = symbol.elements.iter()
        .filter(|e| match e {
            SymbolElement::PART(_) => true,
            _ => false,
        })
        .count() > 1;
    let mut index = 1;
    for element in &mut symbol.elements {
        match element {
            SymbolElement::PART(part) => {
                part.id = if is_complex_symbol { format!("test.{}", index) } else { "test".into() };
                index += 1;
            }
            _ => {}
        }
    }

    let kicad_symbol_lib: SymbolLib = symbol.try_into()?;

    let item = kicad_symbol_lib.serialize();
    let tokens = KiCadParser::generate_tokens(&item);
    let sym_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
    //println!("{}", string);

    let kicad_footprint: FootprintLibrary = footprint.try_into()?;
    let item = kicad_footprint.serialize();
    let tokens = KiCadParser::generate_tokens(&item);
    let fp_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);

    //println!("{}", fp_string);
    fs::write("/home/sparky/HardwareProjects/iot-controller/test-library.pretty/test-footprint.kicad_mod", fp_string)?;

    //println!("{}", sym_string);
    fs::write("/home/sparky/HardwareProjects/iot-controller/test-symbol.kicad_sym", sym_string)?;

    println!("All done!");
    Ok(())
}

#[allow(unused)]
fn dev_batch_parse_lcsc_all() -> anyhow::Result<()> {
    let mut last_stats_print = Instant::now();
    let files_in_dir = fs::read_dir("/run/media/sparky/Stuff/LCSC/comp_data")?;
    let file_count = fs::read_dir("/run/media/sparky/Stuff/LCSC/comp_data")?.count();
    let mut processed_files = 0;
    let mut processed_bytes = 0;
    let mut successful_files = 0;

    for entry in files_in_dir {
        let entry = entry?;
        let path = entry.path();
        let lcsc_id = path.file_stem().unwrap().to_str().unwrap();
        let data = fs::read_to_string(path.clone())?;
        let data = data.trim();
        if data.len() == 0 {
            continue;
        }

        processed_bytes += data.len();

        let data = serde_json::from_str::<Value>(&data)?;

        let symbol_data = data["symbol"].as_str();
        let footprint_data = data["footprint"].as_str();

        if let (Some(symbol_data), Some(footprint_data)) = (symbol_data, footprint_data) {
            let old_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let result = std::panic::catch_unwind(|| {
                let symbol = EasyEDASymbol::parse(symbol_data).unwrap();
                let footprint = EasyEDAFootprint::parse(footprint_data).unwrap();
                (symbol, footprint)
            });
            std::panic::set_hook(old_hook);

            if let Err(error) = result {
                println!("Failed to parse EasyEDA component {}: {:?}", lcsc_id, error.downcast_ref::<String>().unwrap().split('\n').next().unwrap());
                continue;
            }

            let (symbol, footprint) = result.unwrap();

            let old_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let result = std::panic::catch_unwind(|| {
                let symbol: SymbolLib = symbol.try_into().unwrap();
                let footprint: FootprintLibrary = footprint.try_into().unwrap();
                (symbol, footprint)
            });
            std::panic::set_hook(old_hook);

            if let Err(error) = result {
                if let Some(error_string) = error.downcast_ref::<String>() {
                    println!("Failed to convert component to KiCAD {}: {:?}", lcsc_id, error_string);
                } else {
                    panic!("Failed to convert component to KiCAD due to unknown error: {}", lcsc_id);
                }
                continue;
            }

            let (_symbol, _footprint) = result.unwrap();

            successful_files += 1;
        } else {
            println!("Failed to load component data for {}", lcsc_id);
        }

        processed_files += 1;

        if last_stats_print.elapsed().as_secs_f32() >= 1.0 {
            println!("Processed {}/{} files ({} MB/s); success rate: {}/{}",
                     processed_files, file_count, processed_bytes / 1024 / 1024, successful_files, file_count);
            last_stats_print = Instant::now();
            processed_bytes = 0;
        }
    };

    println!("Total success rate: {}/{}", successful_files, processed_files);

    Ok(())
}

#[allow(unused)]
#[post("/footprint")]
async fn handle_footprint(body: web::Bytes) -> HttpResponse {
    let raw_str = String::from_utf8(body.to_vec()).unwrap();

    match EasyEDAFootprint::parse(&raw_str) {
        Ok(footprint) => {
            println!("Footprint parsed successfully");
            let kicad_footprint: FootprintLibrary = footprint.try_into().unwrap();
            let item = kicad_footprint.serialize();
            let tokens = KiCadParser::generate_tokens(&item);
            let string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
            // println!("{}", string);

            fs::write("/home/sparky/HardwareProjects/iot-controller/test-library.pretty/test-footprint.kicad_mod", string).unwrap();

            println!("All done!");
        }
        Err(err) => {
            println!("Footprint parse error: {}", err);
        }
    }

    HttpResponse::Ok().await.unwrap()
}

#[allow(unused)]
#[get("/svgs/{lcsc}")]
async fn handle_svgs_conversion(path: web::Path<String>) -> impl Responder {
    match process_conversion(path.to_string()) {
        Ok(result) => HttpResponse::Ok()
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .body(result),
        Err(err) => HttpResponse::InternalServerError()
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .body(err.to_string()),
    }
}

#[allow(unused)]
fn process_conversion(code: String) -> anyhow::Result<String> {
    println!("Processing request for {}", code);

    let (mut symbol, footprint) = easyeda::tests::download_component(code.as_str())?; // C3682882
    let is_complex_symbol = symbol.elements.iter()
        .filter(|e| match e {
            SymbolElement::PART(_) => true,
            _ => false,
        })
        .count() > 1;
    let mut index = 1;
    for element in &mut symbol.elements {
        match element {
            SymbolElement::PART(part) => {
                part.id = if is_complex_symbol { format!("test.{}", index) } else { "test".into() };
                index += 1;
            }
            _ => {}
        }
    }

    let mut kicad_symbol_lib: SymbolLib = symbol.try_into()?;

    let item = kicad_symbol_lib.serialize();
    let tokens = KiCadParser::generate_tokens(&item);
    let sym_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
    fs::write("/home/sparky/HardwareProjects/iot-controller/test-symbol.kicad_sym", &sym_string)?;

    let kicad_footprint: FootprintLibrary = footprint.try_into()?;
    let item = kicad_footprint.serialize();
    let tokens = KiCadParser::generate_tokens(&item);
    let fp_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
    fs::write("/home/sparky/HardwareProjects/iot-controller/test-library.pretty/test-footprint.kicad_mod", &fp_string)?;

    let tmp_path = "/tmp/lcsc_web_converter";
    if !fs::exists(tmp_path)? {
        fs::create_dir(tmp_path)?;
    }

    fs::write(format!("{}/test-component.kicad_sym", tmp_path), sym_string)?;
    fs::write(format!("{}/test-component.kicad_mod", tmp_path), fp_string)?;

    // Plot symbol
    let symbol_plot_output = Command::new("kicad-cli")
        .args(["sym", "export", "svg", "-o", tmp_path, format!("{}/test-component.kicad_sym", tmp_path).as_str()])
        .output()
        .expect("Failed to execute kicad-cli to export the symbol");
    let proc_output = String::from_utf8(symbol_plot_output.stdout)?;
    let re = Regex::new(r"Plotting symbol '.+?' to '(.+?)'")?;
    let result = re.captures(proc_output.as_str()).unwrap();
    let output_path = result.get(1).unwrap().as_str();
    let kicad_sym_svg = fs::read_to_string(output_path)?;

    // Plot footprint
    let symbol_plot_output = Command::new("kicad-cli")
        .args(["fp", "export", "svg", "-l", "F.Fab,F.Courtyard,F.Mask,B.Mask,F.Silkscreen,F.Paste,F.Adhesive,F.Cu", "-o", tmp_path, format!("{}/", tmp_path).as_str()])
        .output()
        .expect("Failed to execute kicad-cli to export the symbol");
    let proc_output = String::from_utf8(symbol_plot_output.stdout)?;
    let re = Regex::new(r"Plotting footprint '.+?' to '(.+?)'")?;
    let result = re.captures(proc_output.as_str()).unwrap();
    let output_path = result.get(1).unwrap().as_str();
    let kicad_fp_svg = fs::read_to_string(output_path)?;

    let value = json!({
        "symbol": kicad_sym_svg.as_str(),
        "footprint": kicad_fp_svg.as_str(),
    });

    fs::remove_dir_all(tmp_path)?;

    Ok(value.to_string())
}

#[allow(unused)]
async fn dev_conversion_server() -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(handle_footprint)
            .service(handle_svgs_conversion)
    }).bind(("127.0.0.1", 8088))?.run().await?;

    Ok(())
}