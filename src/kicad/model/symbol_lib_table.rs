use crate::kicad::syntax::{PositionPreference, SyntaxArgument, SyntaxItem, SyntaxItemSerializable};

#[derive(Debug, Default)]
pub struct SymbolLibTable {
    pub version: u8,
    pub libraries: Vec<SymbolLibTableItem>,
}

#[derive(Debug, Default)]
pub struct SymbolLibTableItem {
    pub name: String,
    pub uri: String,
    pub lib_type: String,
    pub options: String,
    pub description: String,
    pub disabled: bool,
    pub hidden: bool,
}

impl SyntaxItemSerializable for SymbolLibTable {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("version", SyntaxArgument::Number(self.version as f32, PositionPreference::None)),
        ];

        for item in &self.libraries {
            children.push(item.serialize());
        }

        SyntaxItem {
            name: "sym_lib_table".into(),
            arguments: vec![],
            children,
        }
    }

    fn deserialize(syntax: &SyntaxItem) -> Self {
        Self {
            version: syntax.get_named_child("version").unwrap().arguments.first().unwrap().get_number() as u8,
            libraries: syntax.children.iter()
                .filter(|i| i.name == "lib")
                .map(|i| SymbolLibTableItem::deserialize(i))
                .collect()
        }
    }
}

impl SyntaxItemSerializable for SymbolLibTableItem {
    fn serialize(&self) -> SyntaxItem {
        let mut children = vec![
            SyntaxItem::from_single_argument("name", SyntaxArgument::QuotedString(self.name.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("uri", SyntaxArgument::QuotedString(self.uri.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("type", SyntaxArgument::QuotedString(self.lib_type.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("options", SyntaxArgument::QuotedString(self.options.clone(), PositionPreference::None)),
            SyntaxItem::from_single_argument("description", SyntaxArgument::QuotedString(self.description.clone(), PositionPreference::None)),
        ];

        if self.disabled {
            children.push(SyntaxItem::from_arguments("disabled", vec![]));
        }

        if self.hidden {
            children.push(SyntaxItem::from_arguments("hidden", vec![]));
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
            disabled: syntax.get_named_child("disabled").unwrap().arguments.first().is_some(),
            hidden: syntax.get_named_child("hidden").unwrap().arguments.first().is_some()
        }
    }
}