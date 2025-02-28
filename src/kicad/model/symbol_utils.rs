use crate::kicad::model::common::{Position, TextEffect};
use crate::kicad::model::symbol_library::{Property, Symbol};

impl Symbol {
    pub fn add_hidden_property(&mut self, key: &str, value: &str) {
        let mut text_effect = TextEffect::default();
        text_effect.hide = true;

        self.properties.push(Property {
            id: None,
            key: key.into(),
            value: value.into(),
            position: Position {
                x: 0.0,
                y: 0.0,
                angle: Some(0.0),
            },
            text_effects: text_effect,
            hide: false,
        });
    }

    pub fn add_property(&mut self, key: &str, value: &str, x: f32, y: f32) {
        self.properties.push(Property {
            id: None,
            key: key.into(),
            value: value.into(),
            position: Position {
                x,
                y,
                angle: Some(0.0),
            },
            text_effects: TextEffect::default(),
            hide: false,
        });
    }
}