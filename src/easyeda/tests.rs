use crate::easyeda::footprint::EasyEDAFootprint;
use crate::easyeda::symbol::EasyEDASymbol;

#[allow(unused)]
macro_rules! test_component {
    ($test_name:ident, $lcsc_code:expr) => {
        #[test]
        fn $test_name() -> anyhow::Result<()> {
            let (symbol, footprint) = easyeda::tests::download_component($lcsc_code)?;

            let kicad_symbol_lib: SymbolLib = symbol.into();
            let item = kicad_symbol_lib.serialize();
            let tokens = KiCadParser::generate_tokens(&item);
            let sym_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);

            let kicad_footprint: FootprintLibrary = footprint.into();
            let item = kicad_footprint.serialize();
            let tokens = KiCadParser::generate_tokens(&item);
            let fp_string = KiCadParser::stringify_tokens::<SymbolLib>(&tokens);

            Ok(())
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::easyeda;
    use crate::kicad::model::footprint_library::FootprintLibrary;
    use crate::kicad::model::symbol_library::SymbolLib;
    use crate::kicad::syntax::{KiCadParser, SyntaxItemSerializable};

    test_component!(stm32_l1, "C165948");
    test_component!(esp32_s3_wroom1, "C2913204");
    test_component!(usb_c_conn_1, "C2765186");
    test_component!(usb_c_conn_2, "C165948");
    test_component!(hdmi_conn_1, "C136421");
    test_component!(microsd_socket, "C105419");
    test_component!(sdram_ic, "C2803250");
    test_component!(rgb_led_1, "C5446699");
    test_component!(rgb_led_2, "C2874116");
    test_component!(rgb_led_3, "C19171339");
    test_component!(resistor, "C21190");
    test_component!(mosfet1, "C18191948");
    test_component!(mosfet2, "C8545");
    test_component!(relay_1, "C93168");
    test_component!(relay_2, "C968857");
    test_component!(relay_3, "C2753312");
    test_component!(relay_4, "C192257");
    test_component!(relay_5, "C23510");
    test_component!(screw_term_5p, "C474923");
    test_component!(screw_term_2p, "C474920");
    test_component!(tactile_button_1, "C318884");
    test_component!(stepdown_converter_1, "C841386");
    test_component!(w_pdfn_8_ep, "C2065307");
    test_component!(son_8, "C3680787");
    test_component!(sod_323, "C2919018");
    test_component!(fcqfn_10, "C506188");
    test_component!(ubga_484, "C1553553");
    test_component!(tqfp_144, "C14689");
    test_component!(sot_89_3, "C9634");
    test_component!(dfn_20l, "C610274");
    test_component!(lfpak, "C134470");
    test_component!(lfpak_56_5, "C503614");
    test_component!(ybs, "C698599");
    test_component!(dfn_20_ep, "C509050"); // fail; unwrap on None value
    test_component!(hclga_4ld, "C2688664"); // fail; unwrap on None value
    test_component!(v_dfn3030_8k, "C155503"); // fail; unwrap on None value
    test_component!(unk_1, "C3032566"); // fail; Unwrap on None in parse_path_expression / "R" / corner_radius
}

pub fn download_component(code: &str) -> anyhow::Result<(EasyEDASymbol, EasyEDAFootprint)> {
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