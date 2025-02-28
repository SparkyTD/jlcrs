use crate::kicad::syntax::{PositionPreference, SyntaxArgument, SyntaxItem, SyntaxItemSerializable, TopLevelSerializable};

#[derive(Debug, Default)]
pub struct FootprintLibTable {
    pub version: u8,
    pub libraries: Vec<FootprintLibTableItem>,
}

#[derive(Debug, Default)]
pub struct FootprintLibTableItem {
    pub name: String,
    pub uri: String,
    pub lib_type: String,
    pub options: String,
    pub description: String,
    pub disabled: bool,
}

impl TopLevelSerializable for FootprintLibTable {
    fn get_same_line_identifiers() -> Vec<String> {
        Vec::from([
            "name", "type", "uri", "options", "descr", "disabled", "hidden",
        ]).iter().map(|s| s.to_string()).collect()
    }
}

impl SyntaxItemSerializable for FootprintLibTable {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("version", SyntaxArgument::Number(self.version as f32, PositionPreference::None)),
        ];

        for item in &self.libraries {
            children.push(item.serialize());
        }

        SyntaxItem {
            name: "fp_lib_table".into(),
            arguments: vec![],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            version: syntax.get_named_child("version").unwrap().arguments.first().unwrap().get_number() as u8,
            libraries: syntax.children.iter()
                .filter(|i| i.name == "lib")
                .map(|i| FootprintLibTableItem::deserialize(i))
                .collect()
        }
    }
}

impl SyntaxItemSerializable for FootprintLibTableItem {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("name", SyntaxArgument::QuotedString(self.name.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("type", SyntaxArgument::QuotedString(self.lib_type.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("uri", SyntaxArgument::QuotedString(self.uri.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("options", SyntaxArgument::QuotedString(self.options.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("descr", SyntaxArgument::QuotedString(self.description.clone(), PositionPreference::None)),
        ];

        if self.disabled {
            children.push(SyntaxItem::from_arguments("disabled", vec![]));
        }

        SyntaxItem {
            name: "lib".into(),
            arguments: vec![],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            name: syntax.get_named_child("name").unwrap().arguments.first().unwrap().get_string(),
            uri: syntax.get_named_child("uri").unwrap().arguments.first().unwrap().get_string(),
            lib_type: syntax.get_named_child("type").unwrap().arguments.first().unwrap().get_string(),
            options: syntax.get_named_child("options").unwrap().arguments.first().unwrap().get_string(),
            description: syntax.get_named_child("descr").unwrap().arguments.first().unwrap().get_string(),
            disabled: syntax.get_named_child("disabled").is_some(),
        }
    }
}