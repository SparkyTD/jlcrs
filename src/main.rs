use crate::easyeda::footprint::EasyEDAFootprint;
use crate::easyeda::symbol::SymbolElement;
use crate::kicad::model::symbol_library::SymbolLib;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use kicad::model::footprint_library::FootprintLibrary;
use kicad::syntax::SyntaxItemSerializable;
use kicad::syntax::{KiCadParser, TopLevelSerializable};
use proc_macro2::TokenStream;
use std::fs;
use svg::node::NodeClone;

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

fn main7() -> anyhow::Result<()> {
    let test_input_sym = fs::read_to_string("/home/sparky/HardwareProjects/iot-controller/lfxp2-5e-5tn144.kicad_sym")?;

    test_parse_file::<SymbolLib>(&test_input_sym)?;

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
    let (mut symbol, footprint) = easyeda::tests::download_component("C136421")?;

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

    let mut kicad_symbol_lib: SymbolLib = symbol.into();

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

    //println!("{}", sym_string);
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

#[tokio::main]
async fn main3() -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new().service(handle_footprint)
    }).bind(("127.0.0.1", 8089))?.run().await?;

    Ok(())
}