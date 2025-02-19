use std::fs;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use crate::easyeda::symbol::EasyEDASymbol;
use kicad::model::footprint_library::FootprintLibrary;
use kicad::syntax::SyntaxItemSerializable;
use kicad::syntax::{KiCadParser, TopLevelSerializable};
use proc_macro2::TokenStream;
use svg::node::NodeClone;
use crate::easyeda::footprint::EasyEDAFootprint;
use crate::kicad::model::symbol_library::SymbolLib;

mod kicad;
mod easyeda;

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

pub fn test_derive(input: TokenStream) -> TokenStream {
    input
}

fn main1() -> anyhow::Result<()> {
    // /home/sparky/Downloads/HF49FD_024-1H12T/KiCad/HF49FD0241H12T.kicad_mod

    let all_files = fs::read_dir("/home/sparky/Downloads/JLCPCB-KiCad-Library-2025.01.22/footprints/JLCPCB.pretty")?;
    // let all_files = fs::read_dir("/home/sparky/HardwareProjects/iot-controller/library/snapeda/Footprints.pretty")?;
    for entry in all_files.map(|f| f.unwrap()) {
        if entry.path().extension().unwrap().to_str().unwrap() != "kicad_mod" {
            continue;
        }

        let test_input = fs::read_to_string(&entry.path())?;
        println!("Testing: {}", entry.path().display());
        test_parse_file::<FootprintLibrary>(&test_input)?;
    }

    //test_parse_file::<SymbolLib>(&test_input)?;
    //test_parse_file::<FootprintLibrary>(&test_input)?;

    Ok(())
}


fn main() -> anyhow::Result<()> {
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
    let (symbol, footprint) = download_component("C165948")?;

    let kicad_symbol_lib: SymbolLib = symbol.into();
    let item = kicad_symbol_lib.serialize();
    let tokens = KiCadParser::generate_tokens(&item);
    let sym_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
    //println!("{}", string);

    let kicad_footprint: FootprintLibrary = footprint.into();
    let item = kicad_footprint.serialize();
    let tokens = KiCadParser::generate_tokens(&item);
    let fp_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);

    //println!("{}", fp_string);
    fs::write("/home/sparky/HardwareProjects/iot-controller/test-library.pretty/test-footprint.kicad_mod", fp_string)?;

    println!("{}", sym_string);
    fs::write("/home/sparky/HardwareProjects/iot-controller/test-symbol.kicad_sym", sym_string)?;

    println!("All done!");
    Ok(())
}

#[post("/footprint")]
async fn handle_footprint(body: web::Bytes) -> HttpResponse {
    let raw_str = String::from_utf8(body.to_vec()).unwrap();

    match EasyEDAFootprint::parse(&raw_str) {
        Ok(footprint) => {
            println!("Footprint parsed successfully");
            let kicad_footprint: FootprintLibrary = footprint.into();
            let item = kicad_footprint.serialize();
            let tokens = KiCadParser::generate_tokens(&item);
            let string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);
            println!("{}", string);

            fs::write("/home/sparky/HardwareProjects/iot-controller/test-library.pretty/test-footprint.kicad_mod", string).unwrap();

            println!("All done!");

        }
        Err(err) => {
            println!("Footprint parse error: {}", err);
        }
    }

    HttpResponse::Ok().await.unwrap()
}

//#[tokio::main]
async fn main2() -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new().service(handle_footprint)
    }).bind(("127.0.0.1", 8088))?.run().await?;

    Ok(())
}

fn download_component(code: &str) -> anyhow::Result<(EasyEDASymbol, EasyEDAFootprint)> {
    let response = ureq::get(
        format!("https://pro.easyeda.com/api/eda/product/search?keyword={}&currPage=1&pageSize=1", code)
    ).call()?;

    let body_string = response.into_body().read_to_string()?;
    let json = serde_json::from_str::<serde_json::Value>(&body_string)?;
    let data = &json["result"]["productList"][0]["device_info"];
    let mut symbol = EasyEDASymbol::parse(&data["symbol_info"]["dataStr"].as_str().unwrap())?;
    let mut footprint = EasyEDAFootprint::parse(&data["footprint_info"]["dataStr"].as_str().unwrap())?;

    symbol.part_number = Some(code.into());
    footprint.part_number = Some(code.into());

    Ok((symbol, footprint))
}