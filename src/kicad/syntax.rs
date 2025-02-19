use std::collections::VecDeque;
use itertools::Itertools;

#[derive(Debug, PartialEq)]
pub enum Token {
    OpenParen(usize),
    CloseParen(usize),
    Identifier(usize, String),
    QuotedString(usize, String),
    Number(usize, f32),
}

impl Token {
    #[allow(dead_code)]
    pub fn is_opening_paren(&self) -> bool {
        if let Token::OpenParen(_) = self {
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn is_closing_paren(&self) -> bool {
        if let Token::CloseParen(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxItem {
    pub name: String,
    pub arguments: Vec<SyntaxArgument>,
    pub children: Vec<SyntaxItem>,
}

impl SyntaxItem {
    pub fn from_single_argument(name: &str, argument: SyntaxArgument) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            arguments: vec![argument],
        }
    }

    pub fn from_single_child(name: &str, child: SyntaxItem) -> Self {
        Self {
            name: name.into(),
            children: vec![child],
            arguments: Vec::new(),
        }
    }

    pub fn from_arguments(name: &str, arguments: Vec<SyntaxArgument>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            arguments,
        }
    }

    pub fn get_named_child(&self, name: &str) -> Option<&SyntaxItem> {
        self.children.iter().find(|item| item.name == name)
    }

    pub fn has_argument(&self, argument: SyntaxArgument) -> bool {
        self.arguments.iter().find(|a| **a == argument).is_some()
    }

    pub fn deep_equals(&self, other: &SyntaxItem) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.arguments.len() != other.arguments.len() && self.name != "layers" {
            return false;
        }

        let this_children = self.children.iter().sorted_by_key(|e| e.name.clone()).collect_vec();
        let other_children = other.children.iter()
            .sorted_by_key(|e| e.name.clone())
            .filter(|e| e.name != "teardrop")
            .filter(|e| e.name != "thermal_bridge_angle")
            .collect_vec();

        if this_children.len() != other_children.len() {
            println!(">>> Mismatched child count in {}", self.name);
            println!("    self: {:?}", this_children.iter().map(|c| c.name.clone()).collect_vec());
            println!("   other: {:?}", other_children.iter().map(|c| c.name.clone()).collect_vec());
            return false;
        }

        for i in 0..this_children.len() {
            let this = this_children.get(i).unwrap();
            let other = other_children.get(i).unwrap();
            if !this.deep_equals(other) {
                return false;
            }
        }

        if self.name != "layers" {
            for i in 0..self.arguments.len() {
                let this = self.arguments.get(i).unwrap();
                let other = other.arguments.get(i).unwrap();
                if this.get_string() != other.get_string() {
                    if self.name == "fill" && (this.get_string() == "yes" && other.get_string() == "solid" ||
                        this.get_string() == "solid" && other.get_string() == "yes" ||
                        this.get_string() == "no" && other.get_string() == "none" ||
                        this.get_string() == "none" && other.get_string() == "no") {
                        continue;
                    }
                    return false;
                }
            }
        }

        true
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Copy)]
pub enum PositionPreference {
    Start,
    None,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxArgument {
    Number(f32, PositionPreference),
    Identifier(String, PositionPreference),
    QuotedString(String, PositionPreference),
}

impl SyntaxArgument {
    pub fn get_number(&self) -> f32 {
        if let SyntaxArgument::Number(n, _) = self {
            *n
        } else {
            panic!("Not a number")
        }
    }

    pub fn get_string(&self) -> String {
        if let SyntaxArgument::Identifier(str, _) = self {
            str.clone()
        } else if let SyntaxArgument::QuotedString(str, _) = self {
            str.clone()
        } else if let SyntaxArgument::Number(num, _) = self {
            num.to_string()
        } else {
            unreachable!()
        }
    }
}

pub struct KiCadParser;

impl KiCadParser {
    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        let mut position: usize = 0;

        while let Some(&ch) = chars.peek() {
            match ch {
                '\r' | '\n' | ' ' => {
                    chars.next();
                    position += 1;
                    continue;
                }
                '(' => {
                    tokens.push(Token::OpenParen(position));
                    chars.next();
                    position += 1;
                }
                ')' => {
                    tokens.push(Token::CloseParen(position));
                    chars.next();
                    position += 1;
                }
                '"' => {
                    chars.next(); // Skip opening code
                    position += 1;
                    let mut string = String::new();
                    while let Some(ch) = chars.next() {
                        position += 1;
                        if ch == '"' {
                            break;
                        }
                        string.push(ch);
                    }
                    tokens.push(Token::QuotedString(position, string));
                }
                _ if Self::is_char_identifier_or_numeric(ch) => {
                    let mut string = String::new();
                    while let Some(ch) = chars.peek() {
                        if Self::is_char_identifier_or_numeric(*ch) {
                            string.push(*ch);
                            chars.next();
                            position += 1;
                        } else if *ch == ' ' || *ch == ')' || *ch == '\r' || *ch == '\n' {
                            break;
                        } else {
                            panic!("Invalid identifier token at {}!", position);
                        }
                    }
                    if let Ok(number) = string.parse::<f32>() {
                        tokens.push(Token::Number(position, number));
                    } else {
                        tokens.push(Token::Identifier(position, string));
                    }
                }
                _ => {
                    chars.next();
                    position += 1;
                }
            }
        }

        tokens
    }

    pub fn parse_syntax_item(tokens: &Vec<Token>) -> SyntaxItem {
        let mut items = VecDeque::<SyntaxItem>::new();

        for token in tokens.iter().peekable() {
            match token {
                Token::OpenParen(_) => {
                    items.push_front(SyntaxItem {
                        name: "".into(),
                        arguments: Vec::new(),
                        children: Vec::new(),
                    });
                }
                Token::CloseParen(_) => {
                    let current_element = items.pop_front().unwrap();
                    if let Some(parent_element) = items.front_mut() {
                        parent_element.children.push(current_element);
                    } else {
                        items.push_front(current_element);
                    }
                }
                Token::Identifier(offset, str) => {
                    if let Some(top_item) = items.front_mut() {
                        if top_item.name.is_empty() {
                            top_item.name = str.clone();
                        } else {
                            top_item
                                .arguments
                                .push(SyntaxArgument::Identifier(str.clone(), PositionPreference::None));
                        }
                    } else {
                        panic!("It is invalid to have an identifier with no parent node (at offset {})", offset)
                    }
                }
                Token::QuotedString(offset, str) => {
                    if let Some(top_item) = items.front_mut() {
                        top_item
                            .arguments
                            .push(SyntaxArgument::QuotedString(str.clone(), PositionPreference::None));
                    } else {
                        panic!("It is invalid to have a string value with no parent node (at offset {})", offset)
                    }
                }
                Token::Number(offset, val) => {
                    if let Some(top_item) = items.front_mut() {
                        top_item.arguments.push(SyntaxArgument::Number(*val, PositionPreference::None));
                    } else {
                        panic!("It is invalid to have a numeric value with no parent node (at offset {})", offset)
                    }
                }
            }
        }

        items.pop_front().unwrap()
    }

    pub fn generate_tokens(item: &SyntaxItem) -> Vec<Token> {
        let mut tokens = Vec::new();
        tokens.push(Token::OpenParen(0));

        tokens.push(Token::Identifier(0, item.name.clone()));

        let mut content_tokens: Vec<(Token, PositionPreference)> = Vec::new();

        for argument in &item.arguments {
            match argument {
                SyntaxArgument::QuotedString(str, order) => content_tokens.push((Token::QuotedString(0, str.clone()), *order)),
                SyntaxArgument::Identifier(str, order) => content_tokens.push((Token::Identifier(0, str.clone()), *order)),
                SyntaxArgument::Number(val, order) => content_tokens.push((Token::Number(0, *val), *order)),
            }
        }

        for child in &item.children {
            let child_tokens = Self::generate_tokens(child);
            for token in child_tokens {
                content_tokens.push((token, PositionPreference::None));
            }
        }

        content_tokens.sort_by(|a, b| a.1.cmp(&b.1));
        tokens.extend(content_tokens.into_iter().map(|(tok, _)| tok).collect::<Vec<_>>());

        tokens.push(Token::CloseParen(0));
        tokens
    }

    pub fn stringify_tokens<S>(tokens: &Vec<Token>) -> String
    where
        S: TopLevelSerializable,
    {
        let mut string = String::new();
        let mut tokens = tokens.iter().peekable();
        let mut indent = 0;
        let mut last_token_is_closing_paren = false;
        let mut identifier_stack = VecDeque::new();
        let mut last_popped_item_name: Option<String> = None;

        let same_line_identifiers = S::get_same_line_identifiers();
        let effects_same_line_after = ["name", "number"];
        while let Some(token) = tokens.next() {
            let same_line = match (token, tokens.peek()) {
                (Token::OpenParen(_), Some(Token::Identifier(_, str))) => {
                    identifier_stack.push_front(str.clone());
                    same_line_identifiers.contains(&str)
                }
                _ => false
            };

            let top_item_name = identifier_stack.front();

            match token {
                Token::OpenParen(_) => {
                    let mut force_skip_new_line = false;
                    if let (Some(Token::Identifier(_, str)), Some(previous_item)) = (tokens.peek(), identifier_stack.get(1)) {
                        if str == "effects" && effects_same_line_after.contains(&previous_item.as_str()) {
                            force_skip_new_line = true;
                        }
                    }

                    if !force_skip_new_line & &!same_line {
                        if string.ends_with(' ') {
                            string.pop();
                        }

                        string.push('\n');
                        string.push_str(&" ".repeat(indent * 2));
                    }
                    indent += 1;

                    string.push('(');
                }
                Token::CloseParen(_) => {
                    indent -= 1;
                    let mut force_skip_new_line = false;
                    if last_token_is_closing_paren {
                        if top_item_name.is_some_and(|f| f == "effects") {
                            if effects_same_line_after.contains(&identifier_stack.get(1).unwrap().as_str()) {
                                force_skip_new_line = true;
                            }
                        }

                        if let (Some(top_item_name), Some(last_popped_item_name)) = (top_item_name, last_popped_item_name.as_ref()) {
                            if last_popped_item_name == "effects" && effects_same_line_after.contains(&top_item_name.as_str()) {
                                force_skip_new_line = true;
                            }

                            if same_line_identifiers.contains(&last_popped_item_name) {
                                force_skip_new_line = true;
                            }
                        }

                        if !force_skip_new_line & &!same_line_identifiers.contains(&top_item_name.unwrap()) {
                            string.push('\n');
                            string.push_str(&" ".repeat(indent * 2));
                        }
                    }

                    string.push(')');

                    last_popped_item_name = identifier_stack.pop_front();

                    if tokens.peek().is_some_and(|&t| !t.is_closing_paren()) {
                        string.push(' ');
                    }
                }
                Token::QuotedString(_, str) => {
                    string.push_str(format!("\"{}\"", str).as_str());
                    if tokens.peek().is_some_and(|&t| !t.is_closing_paren()) {
                        string.push(' ');
                    }
                }
                Token::Identifier(_, str) => {
                    string.push_str(str.as_str());
                    if tokens.peek().is_some_and(|&t| !t.is_closing_paren()) {
                        string.push(' ');
                    }
                }
                Token::Number(_, val) => {
                    string.push_str(format!("{}", val).as_str());
                    if tokens.peek().is_some_and(|&t| !t.is_closing_paren()) {
                        string.push(' ');
                    }
                }
            }

            match token {
                Token::CloseParen(_) => last_token_is_closing_paren = true,
                _ => last_token_is_closing_paren = false,
            }
        }

        string
    }

    fn is_char_identifier_or_numeric(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == '*' || ch == '%'
    }
}

pub trait SyntaxItemSerializable {
    fn serialize(&self) -> SyntaxItem;
    fn deserialize(syntax: &SyntaxItem) -> Self;
}

pub trait TopLevelSerializable: SyntaxItemSerializable {
    fn get_same_line_identifiers() -> Vec<String>;
}