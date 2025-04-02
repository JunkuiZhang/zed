use collections::HashMap;

use super::{
    always_use_command_layout, chars_for_modified_key, keyboard_layout, KeyCode, Modifiers,
};

/// TODO:
pub struct KeyboardMapperManager {
    mapper: HashMap<String, KeyboardMapper>,
}

/// TODO:
pub struct KeyboardMapper {
    letter: HashMap<String, KeyCode>,
    other: HashMap<String, (KeyCode, Modifiers)>,
    code_to_char: HashMap<KeyCode, String>,
}

impl KeyboardMapperManager {
    pub(crate) fn new() -> Self {
        let mut mapper = HashMap::default();
        let current_layout = keyboard_layout();
        mapper.insert(current_layout, KeyboardMapper::new());

        Self { mapper }
    }

    pub(crate) fn update(&mut self, layout: &str) {
        if !self.mapper.contains_key(layout) {
            let info = KeyboardMapper::new();
            self.mapper.insert(layout.to_string(), info);
        }
    }

    pub(crate) fn get_mapper(&self, layout: &str) -> &KeyboardMapper {
        self.mapper.get(layout).unwrap()
    }
}

impl KeyboardMapper {
    fn new() -> Self {
        let mut letter = HashMap::default();
        let mut other = HashMap::default();
        let mut code_to_char = HashMap::default();

        if always_use_command_layout() {
            letter.insert("a".to_string(), KeyCode::A);
            letter.insert("b".to_string(), KeyCode::B);
            letter.insert("c".to_string(), KeyCode::C);
            letter.insert("d".to_string(), KeyCode::D);
            letter.insert("e".to_string(), KeyCode::E);
            letter.insert("f".to_string(), KeyCode::F);
            letter.insert("g".to_string(), KeyCode::G);
            letter.insert("h".to_string(), KeyCode::H);
            letter.insert("i".to_string(), KeyCode::I);
            letter.insert("j".to_string(), KeyCode::J);
            letter.insert("k".to_string(), KeyCode::K);
            letter.insert("l".to_string(), KeyCode::L);
            letter.insert("m".to_string(), KeyCode::M);
            letter.insert("n".to_string(), KeyCode::N);
            letter.insert("o".to_string(), KeyCode::O);
            letter.insert("p".to_string(), KeyCode::P);
            letter.insert("q".to_string(), KeyCode::Q);
            letter.insert("r".to_string(), KeyCode::R);
            letter.insert("s".to_string(), KeyCode::S);
            letter.insert("t".to_string(), KeyCode::T);
            letter.insert("u".to_string(), KeyCode::U);
            letter.insert("v".to_string(), KeyCode::V);
            letter.insert("w".to_string(), KeyCode::W);
            letter.insert("x".to_string(), KeyCode::X);
            letter.insert("y".to_string(), KeyCode::Y);
            letter.insert("z".to_string(), KeyCode::Z);
        }

        for (scan_code, code) in ALL_CODES {
            for (key, modifiers) in generate_keymap_info(scan_code) {
                if modifiers == Modifiers::none() {
                    code_to_char.insert(code, key.clone());
                }
                other.insert(key, (code, modifiers));
            }
        }

        Self {
            letter,
            other,
            code_to_char,
        }
    }

    pub(crate) fn parse(&self, input: &str, char_matching: bool) -> Option<(KeyCode, Modifiers)> {
        if !char_matching {
            if let Some(code) = self.letter.get(input) {
                return Some((*code, Modifiers::none()));
            }
            if let Some(code) = match input {
                "0" => Some(KeyCode::Digital0),
                "1" => Some(KeyCode::Digital1),
                "2" => Some(KeyCode::Digital2),
                "3" => Some(KeyCode::Digital3),
                "4" => Some(KeyCode::Digital4),
                "5" => Some(KeyCode::Digital5),
                "6" => Some(KeyCode::Digital6),
                "7" => Some(KeyCode::Digital7),
                "8" => Some(KeyCode::Digital8),
                "9" => Some(KeyCode::Digital9),
                ";" => Some(KeyCode::Semicolon),
                "=" => Some(KeyCode::Plus),
                "," => Some(KeyCode::Comma),
                "-" => Some(KeyCode::Minus),
                "." => Some(KeyCode::Period),
                "/" => Some(KeyCode::Slash),
                "`" => Some(KeyCode::Tilde),
                "[" => Some(KeyCode::LeftBracket),
                "\\" => Some(KeyCode::Backslash),
                "]" => Some(KeyCode::RightBracket),
                "'" => Some(KeyCode::Quote),
                _ => None,
            } {
                return Some((code, Modifiers::none()));
            }
        } else {
            if let Some((code, modifiers)) = self.other.get(input) {
                return Some((*code, *modifiers));
            }
            if let Some(code) = self.letter.get(input) {
                return Some((*code, Modifiers::none()));
            }
        }
        None
    }

    pub(crate) fn code_to_char(&self, code: KeyCode) -> Option<String> {
        self.code_to_char.get(&code).cloned()
    }
}

fn generate_keymap_info(scan_code: u16) -> Vec<(String, Modifiers)> {
    let mut keymap = Vec::new();
    let no_mod = chars_for_modified_key(scan_code, NO_MOD);
    if !no_mod.is_empty() {
        keymap.push((no_mod, Modifiers::none()));
    }
    let shift_mod = chars_for_modified_key(scan_code, SHIFT_MOD);
    if !shift_mod.is_empty() {
        keymap.push((shift_mod, Modifiers::shift()));
    }
    let alt_mod = chars_for_modified_key(scan_code, OPTION_MOD);
    if !alt_mod.is_empty() {
        keymap.push((alt_mod, Modifiers::alt()));
    }
    let shift_alt_mod = chars_for_modified_key(scan_code, SHIFT_MOD | OPTION_MOD);
    if !shift_alt_mod.is_empty() {
        keymap.push((
            shift_alt_mod,
            Modifiers {
                shift: true,
                alt: true,
                ..Default::default()
            },
        ));
    }
    keymap
}

const NO_MOD: u32 = 0;
const SHIFT_MOD: u32 = 2;
const OPTION_MOD: u32 = 8;

static ALL_CODES: [(u16, KeyCode); 47] = [
    // 0x001d => KeyCode::Digital0,
    (0x001d, KeyCode::Digital0),
    // 0x0012 => KeyCode::Digital1,
    (0x0012, KeyCode::Digital1),
    // 0x0013 => KeyCode::Digital2,
    (0x0013, KeyCode::Digital2),
    // 0x0014 => KeyCode::Digital3,
    (0x0014, KeyCode::Digital3),
    // 0x0015 => KeyCode::Digital4,
    (0x0015, KeyCode::Digital4),
    // 0x0017 => KeyCode::Digital5,
    (0x0017, KeyCode::Digital5),
    // 0x0016 => KeyCode::Digital6,
    (0x0016, KeyCode::Digital6),
    // 0x001a => KeyCode::Digital7,
    (0x001a, KeyCode::Digital7),
    // 0x001c => KeyCode::Digital8,
    (0x001c, KeyCode::Digital8),
    // 0x0019 => KeyCode::Digital9,
    (0x0019, KeyCode::Digital9),
    // 0x0029 => KeyCode::Semicolon,
    (0x0029, KeyCode::Semicolon),
    // 0x0018 => KeyCode::Plus,
    (0x0018, KeyCode::Plus),
    // 0x002b => KeyCode::Comma,
    (0x002b, KeyCode::Comma),
    // 0x001b => KeyCode::Minus,
    (0x001b, KeyCode::Minus),
    // 0x002f => KeyCode::Period,
    (0x002f, KeyCode::Period),
    // 0x002c => KeyCode::Slash,
    (0x002c, KeyCode::Slash),
    // 0x0032 => KeyCode::Tilde,
    (0x0032, KeyCode::Tilde),
    // 0x0021 => KeyCode::LeftBracket,
    (0x0021, KeyCode::LeftBracket),
    // 0x002a => KeyCode::Backslash,
    (0x002a, KeyCode::Backslash),
    // 0x001e => KeyCode::RightBracket,
    (0x001e, KeyCode::RightBracket),
    // 0x0027 => KeyCode::Quote,
    (0x0027, KeyCode::Quote),
    // 0x0000 => KeyCode::A,
    (0x0000, KeyCode::A),
    // 0x000b => KeyCode::B,
    (0x000b, KeyCode::B),
    // 0x0008 => KeyCode::C,
    (0x0008, KeyCode::C),
    // 0x0002 => KeyCode::D,
    (0x0002, KeyCode::D),
    // 0x000e => KeyCode::E,
    (0x000e, KeyCode::E),
    // 0x0003 => KeyCode::F,
    (0x0003, KeyCode::F),
    // 0x0005 => KeyCode::G,
    (0x0005, KeyCode::G),
    // 0x0004 => KeyCode::H,
    (0x0004, KeyCode::H),
    // 0x0022 => KeyCode::I,
    (0x0022, KeyCode::I),
    // 0x0026 => KeyCode::J,
    (0x0026, KeyCode::J),
    // 0x0028 => KeyCode::K,
    (0x0028, KeyCode::K),
    // 0x0025 => KeyCode::L,
    (0x0025, KeyCode::L),
    // 0x002e => KeyCode::M,
    (0x002e, KeyCode::M),
    // 0x002d => KeyCode::N,
    (0x002d, KeyCode::N),
    // 0x001f => KeyCode::O,
    (0x001f, KeyCode::O),
    // 0x0023 => KeyCode::P,
    (0x0023, KeyCode::P),
    // 0x000c => KeyCode::Q,
    (0x000c, KeyCode::Q),
    // 0x000f => KeyCode::R,
    (0x000f, KeyCode::R),
    // 0x0001 => KeyCode::S,
    (0x0001, KeyCode::S),
    // 0x0011 => KeyCode::T,
    (0x0011, KeyCode::T),
    // 0x0020 => KeyCode::U,
    (0x0020, KeyCode::U),
    // 0x0009 => KeyCode::V,
    (0x0009, KeyCode::V),
    // 0x000d => KeyCode::W,
    (0x000d, KeyCode::W),
    // 0x0007 => KeyCode::X,
    (0x0007, KeyCode::X),
    // 0x0010 => KeyCode::Y,
    (0x0010, KeyCode::Y),
    // 0x0006 => KeyCode::Z,
    (0x0006, KeyCode::Z),
];
