use std::sync::LazyLock;

use crate::{keyboard_layouts::KeyboardLayout, Keystroke, PlatformKeyboard};

use super::{events::key_string_from_keycode, retrieve_current_keboard_layout};

static KEYBOARD_LAYOUT: LazyLock<KeyboardLayout> =
    LazyLock::new(|| retrieve_current_keboard_layout());

pub(crate) struct MacKeyboard {
    // keyboard:
}

impl PlatformKeyboard for MacKeyboard {
    fn code_to_key(&self, code: &crate::KeyCodes) -> String {
        let keycode = match code {
            crate::KeyCodes::Unknown => 0xFF,
            crate::KeyCodes::Function => 0x3F,
            crate::KeyCodes::Cancel => todo!(),
            crate::KeyCodes::Backspace => 0x33,
            crate::KeyCodes::Tab => 0x30,
            crate::KeyCodes::Clear => todo!(),
            crate::KeyCodes::Enter => 0x24,
            crate::KeyCodes::Shift(key_position) => match key_position {
                crate::KeyPosition::Right => 0x3C,
                _ => 0x38,
            },
            crate::KeyCodes::Control(key_position) => match key_position {
                crate::KeyPosition::Right => 0x3E,
                _ => 0x3B,
            },
            crate::KeyCodes::Alt(key_position) => match key_position {
                crate::KeyPosition::Right => 0x3D,
                _ => 0x3A,
            },
            crate::KeyCodes::Pause => todo!(),
            crate::KeyCodes::Capital => 0x39,
            crate::KeyCodes::Kana => todo!(),
            crate::KeyCodes::Hangul => todo!(),
            crate::KeyCodes::Junja => todo!(),
            crate::KeyCodes::Final => todo!(),
            crate::KeyCodes::Hanja => todo!(),
            crate::KeyCodes::Kanji => todo!(),
            crate::KeyCodes::Escape => 0x35,
            crate::KeyCodes::Convert => todo!(),
            crate::KeyCodes::Nonconvert => todo!(),
            crate::KeyCodes::Accept => todo!(),
            crate::KeyCodes::ModeChange => todo!(),
            crate::KeyCodes::Space => 0x31,
            crate::KeyCodes::PageUp => 0x74,
            crate::KeyCodes::PageDown => 0x79,
            crate::KeyCodes::End => 0x77,
            crate::KeyCodes::Home => 0x73,
            crate::KeyCodes::Left => 0x7B,
            crate::KeyCodes::Up => 0x7E,
            crate::KeyCodes::Right => 0x7C,
            crate::KeyCodes::Down => 0x7D,
            crate::KeyCodes::Select => todo!(),
            crate::KeyCodes::Print => todo!(),
            crate::KeyCodes::Execute => todo!(),
            crate::KeyCodes::PrintScreen => todo!(),
            crate::KeyCodes::Insert => 0x72, // TODO:
            crate::KeyCodes::Delete => 0x75,
            crate::KeyCodes::Help => todo!(),
            crate::KeyCodes::Digital0 => 0x1D,
            crate::KeyCodes::Digital1 => 0x12,
            crate::KeyCodes::Digital2 => 0x13,
            crate::KeyCodes::Digital3 => 0x14,
            crate::KeyCodes::Digital4 => 0x15,
            crate::KeyCodes::Digital5 => 0x17,
            crate::KeyCodes::Digital6 => 0x16,
            crate::KeyCodes::Digital7 => 0x1A,
            crate::KeyCodes::Digital8 => 0x1C,
            crate::KeyCodes::Digital9 => 0x19,
            crate::KeyCodes::A => 0x00,
            crate::KeyCodes::B => 0x0B,
            crate::KeyCodes::C => 0x08,
            crate::KeyCodes::D => 0x02,
            crate::KeyCodes::E => 0x0E,
            crate::KeyCodes::F => 0x03,
            crate::KeyCodes::G => 0x05,
            crate::KeyCodes::H => 0x04,
            crate::KeyCodes::I => 0x22,
            crate::KeyCodes::J => 0x26,
            crate::KeyCodes::K => 0x28,
            crate::KeyCodes::L => 0x25,
            crate::KeyCodes::M => 0x2E,
            crate::KeyCodes::N => 0x2D,
            crate::KeyCodes::O => 0x1F,
            crate::KeyCodes::P => 0x23,
            crate::KeyCodes::Q => 0x0C,
            crate::KeyCodes::R => 0x0F,
            crate::KeyCodes::S => 0x01,
            crate::KeyCodes::T => 0x11,
            crate::KeyCodes::U => 0x20,
            crate::KeyCodes::V => 0x09,
            crate::KeyCodes::W => 0x0D,
            crate::KeyCodes::X => 0x07,
            crate::KeyCodes::Y => 0x10,
            crate::KeyCodes::Z => 0x06,
            crate::KeyCodes::Platform(key_position) => match key_position {
                crate::KeyPosition::Right => 0x36,
                _ => 0x37,
            },
            crate::KeyCodes::App => todo!(),
            crate::KeyCodes::Sleep => todo!(),
            crate::KeyCodes::Numpad0 => 0x52,
            crate::KeyCodes::Numpad1 => 0x53,
            crate::KeyCodes::Numpad2 => 0x54,
            crate::KeyCodes::Numpad3 => 0x55,
            crate::KeyCodes::Numpad4 => 0x56,
            crate::KeyCodes::Numpad5 => 0x57,
            crate::KeyCodes::Numpad6 => 0x58,
            crate::KeyCodes::Numpad7 => 0x59,
            crate::KeyCodes::Numpad8 => 0x5B,
            crate::KeyCodes::Numpad9 => 0x5C,
            crate::KeyCodes::Multiply => 0x43,
            crate::KeyCodes::Add => 0x45,
            crate::KeyCodes::Separator => 0xFF,
            crate::KeyCodes::Subtract => 0x4E,
            crate::KeyCodes::Decimal => 0x41,
            crate::KeyCodes::Divide => 0x4D,
            crate::KeyCodes::F1 => 0x7A,
            crate::KeyCodes::F2 => 0x78,
            crate::KeyCodes::F3 => 0x63,
            crate::KeyCodes::F4 => 0x76,
            crate::KeyCodes::F5 => 0x60,
            crate::KeyCodes::F6 => 0x61,
            crate::KeyCodes::F7 => 0x62,
            crate::KeyCodes::F8 => 0x64,
            crate::KeyCodes::F9 => 0x65,
            crate::KeyCodes::F10 => 0x6D,
            crate::KeyCodes::F11 => 0x67,
            crate::KeyCodes::F12 => 0x6F,
            crate::KeyCodes::F13 => 0x69,
            crate::KeyCodes::F14 => 0x6B,
            crate::KeyCodes::F15 => 0x71,
            crate::KeyCodes::F16 => 0x6A,
            crate::KeyCodes::F17 => 0x40,
            crate::KeyCodes::F18 => 0x4F,
            crate::KeyCodes::F19 => 0x50,
            crate::KeyCodes::F20 => 0x5A,
            crate::KeyCodes::F21 => todo!(),
            crate::KeyCodes::F22 => todo!(),
            crate::KeyCodes::F23 => todo!(),
            crate::KeyCodes::F24 => todo!(),
            crate::KeyCodes::NumLock => todo!(),
            crate::KeyCodes::ScrollLock => todo!(),
            crate::KeyCodes::BrowserBack => todo!(),
            crate::KeyCodes::BrowserForward => todo!(),
            crate::KeyCodes::BrowserRefresh => todo!(),
            crate::KeyCodes::BrowserStop => todo!(),
            crate::KeyCodes::BrowserSearch => todo!(),
            crate::KeyCodes::BrowserFavorites => todo!(),
            crate::KeyCodes::BrowserHome => todo!(),
            crate::KeyCodes::VolumeMute => 0x4A,
            crate::KeyCodes::VolumeDown => 0x49,
            crate::KeyCodes::VolumeUp => 0x48,
            crate::KeyCodes::MediaNextTrack => todo!(),
            crate::KeyCodes::MediaPrevTrack => todo!(),
            crate::KeyCodes::MediaStop => todo!(),
            crate::KeyCodes::MediaPlayPause => todo!(),
            crate::KeyCodes::LaunchMail => todo!(),
            crate::KeyCodes::LaunchMediaSelect => todo!(),
            crate::KeyCodes::LaunchApp1 => todo!(),
            crate::KeyCodes::LaunchApp2 => todo!(),
            crate::KeyCodes::Semicolon => 0x29,
            crate::KeyCodes::Plus => 0x18,
            crate::KeyCodes::Comma => 0x2B,
            crate::KeyCodes::Minus => 0x1B,
            crate::KeyCodes::Period => 0x2F,
            crate::KeyCodes::Slash => 0x2C,
            crate::KeyCodes::Tilde => 0x32,
            crate::KeyCodes::LeftBracket => 0x21,
            crate::KeyCodes::Backslash => 0x2A,
            crate::KeyCodes::RightBracket => 0x1E,
            crate::KeyCodes::Quote => 0x27,
            crate::KeyCodes::OEM8 => todo!(),
            crate::KeyCodes::OEM102 => todo!(),
            crate::KeyCodes::ProcessKey => todo!(),
            crate::KeyCodes::Packet => todo!(),
            crate::KeyCodes::Attn => todo!(),
            crate::KeyCodes::CrSel => todo!(),
            crate::KeyCodes::ExSel => todo!(),
            crate::KeyCodes::EraseEOF => todo!(),
            crate::KeyCodes::Play => todo!(),
            crate::KeyCodes::Zoom => todo!(),
            crate::KeyCodes::PA1 => todo!(),
            crate::KeyCodes::OEMClear => todo!(),
        };
        if keycode == 0xFF {
            "Unknown".to_string()
        } else {
            // map scan code to string
            key_string_from_keycode(keycode, false, false)
        }
    }

    /// Shortcuts translation happens here.
    fn to_native_keystroke(&self, key_stroke: &mut Keystroke) {
        match *KEYBOARD_LAYOUT {
            KeyboardLayout::ABC => {}
            KeyboardLayout::Czech => {}
            KeyboardLayout::CzechQwerty => {}
            KeyboardLayout::German => {}
            KeyboardLayout::Russian => {}
        }
        key_stroke.label = self.code_to_key(&key_stroke.code);
    }
}
