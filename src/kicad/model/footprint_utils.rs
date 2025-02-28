use crate::kicad::model::common::TextEffect;
use crate::kicad::model::footprint_library::{FootprintLibrary, FootprintProperty, PcbLayer, Scalar3D};

impl FootprintLibrary {
    pub fn add_hidden_property(&mut self, key: &str, value: &str) {
        let mut text_effect = TextEffect::default();
        text_effect.hide = true;

        self.properties.push(FootprintProperty {
            key: key.into(),
            value: Some(value.into()),
            position: Scalar3D::new("at", 0.0, 0.0, 0.0),
            hide: Some(true),
            layer: PcbLayer::FFab,
            uuid: None,
            effects: text_effect,
            unlocked: Some(true),
        });
    }
}