/// The mappings defined in this file where created from reading the alacritty source
use alacritty_terminal::term::TermMode;
use gpui::{KeyCodes, Keystroke};

#[derive(Debug, PartialEq, Eq)]
enum AlacModifiers {
    None,
    Alt,
    Ctrl,
    Shift,
    CtrlShift,
    Other,
}

impl AlacModifiers {
    fn new(ks: &Keystroke) -> Self {
        match (
            ks.modifiers.alt,
            ks.modifiers.control,
            ks.modifiers.shift,
            ks.modifiers.platform,
        ) {
            (false, false, false, false) => AlacModifiers::None,
            (true, false, false, false) => AlacModifiers::Alt,
            (false, true, false, false) => AlacModifiers::Ctrl,
            (false, false, true, false) => AlacModifiers::Shift,
            (false, true, true, false) => AlacModifiers::CtrlShift,
            _ => AlacModifiers::Other,
        }
    }

    fn any(&self) -> bool {
        match &self {
            AlacModifiers::None => false,
            AlacModifiers::Alt => true,
            AlacModifiers::Ctrl => true,
            AlacModifiers::Shift => true,
            AlacModifiers::CtrlShift => true,
            AlacModifiers::Other => true,
        }
    }
}

pub fn to_esc_str(keystroke: &Keystroke, mode: &TermMode, alt_is_meta: bool) -> Option<String> {
    let modifiers = AlacModifiers::new(keystroke);

    // Manual Bindings including modifiers
    let manual_esc_str = match (keystroke.key, &modifiers) {
        //Basic special keys
        (KeyCodes::Tab, AlacModifiers::None) => Some("\x09".to_string()),
        (KeyCodes::Escape, AlacModifiers::None) => Some("\x1b".to_string()),
        (KeyCodes::Enter, AlacModifiers::None) => Some("\x0d".to_string()),
        (KeyCodes::Enter, AlacModifiers::Shift) => Some("\x0d".to_string()),
        (KeyCodes::Backspace, AlacModifiers::None) => Some("\x7f".to_string()),
        //Interesting escape codes
        (KeyCodes::Tab, AlacModifiers::Shift) => Some("\x1b[Z".to_string()),
        (KeyCodes::Backspace, AlacModifiers::Ctrl) => Some("\x08".to_string()),
        (KeyCodes::Backspace, AlacModifiers::Alt) => Some("\x1b\x7f".to_string()),
        (KeyCodes::Backspace, AlacModifiers::Shift) => Some("\x7f".to_string()),
        (KeyCodes::Space, AlacModifiers::Ctrl) => Some("\x00".to_string()),
        (KeyCodes::Home, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[1;2H".to_string())
        }
        (KeyCodes::End, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[1;2F".to_string())
        }
        (KeyCodes::PageUp, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[5;2~".to_string())
        }
        (KeyCodes::PageDown, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[6;2~".to_string())
        }
        (KeyCodes::Home, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOH".to_string())
        }
        (KeyCodes::Home, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[H".to_string())
        }
        (KeyCodes::End, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOF".to_string())
        }
        (KeyCodes::End, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[F".to_string())
        }
        (KeyCodes::Up, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOA".to_string())
        }
        (KeyCodes::Up, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[A".to_string())
        }
        (KeyCodes::Down, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOB".to_string())
        }
        (KeyCodes::Down, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[B".to_string())
        }
        (KeyCodes::Right, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOC".to_string())
        }
        (KeyCodes::Right, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[C".to_string())
        }
        (KeyCodes::Left, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOD".to_string())
        }
        (KeyCodes::Left, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[D".to_string())
        }
        // TODO: correct?
        // ("back", AlacModifiers::None) => Some("\x7f".to_string()),
        // (VirtualKeyCode::Backspace, AlacModifiers::None) => Some("\x7f".to_string()),
        (KeyCodes::Insert, AlacModifiers::None) => Some("\x1b[2~".to_string()),
        (KeyCodes::Delete, AlacModifiers::None) => Some("\x1b[3~".to_string()),
        (KeyCodes::PageUp, AlacModifiers::None) => Some("\x1b[5~".to_string()),
        (KeyCodes::PageDown, AlacModifiers::None) => Some("\x1b[6~".to_string()),
        (KeyCodes::F1, AlacModifiers::None) => Some("\x1bOP".to_string()),
        (KeyCodes::F2, AlacModifiers::None) => Some("\x1bOQ".to_string()),
        (KeyCodes::F3, AlacModifiers::None) => Some("\x1bOR".to_string()),
        (KeyCodes::F4, AlacModifiers::None) => Some("\x1bOS".to_string()),
        (KeyCodes::F5, AlacModifiers::None) => Some("\x1b[15~".to_string()),
        (KeyCodes::F6, AlacModifiers::None) => Some("\x1b[17~".to_string()),
        (KeyCodes::F7, AlacModifiers::None) => Some("\x1b[18~".to_string()),
        (KeyCodes::F8, AlacModifiers::None) => Some("\x1b[19~".to_string()),
        (KeyCodes::F9, AlacModifiers::None) => Some("\x1b[20~".to_string()),
        (KeyCodes::F10, AlacModifiers::None) => Some("\x1b[21~".to_string()),
        (KeyCodes::F11, AlacModifiers::None) => Some("\x1b[23~".to_string()),
        (KeyCodes::F12, AlacModifiers::None) => Some("\x1b[24~".to_string()),
        (KeyCodes::F13, AlacModifiers::None) => Some("\x1b[25~".to_string()),
        (KeyCodes::F14, AlacModifiers::None) => Some("\x1b[26~".to_string()),
        (KeyCodes::F15, AlacModifiers::None) => Some("\x1b[28~".to_string()),
        (KeyCodes::F16, AlacModifiers::None) => Some("\x1b[29~".to_string()),
        (KeyCodes::F17, AlacModifiers::None) => Some("\x1b[31~".to_string()),
        (KeyCodes::F18, AlacModifiers::None) => Some("\x1b[32~".to_string()),
        (KeyCodes::F19, AlacModifiers::None) => Some("\x1b[33~".to_string()),
        (KeyCodes::F20, AlacModifiers::None) => Some("\x1b[34~".to_string()),
        // NumpadEnter, Action::Esc("\n".into());
        //Mappings for caret notation keys
        // TODO:
        (KeyCodes::A, AlacModifiers::Ctrl) => Some("\x01".to_string()), //1
        (KeyCodes::A, AlacModifiers::CtrlShift) => Some("\x01".to_string()), //1
        (KeyCodes::B, AlacModifiers::Ctrl) => Some("\x02".to_string()), //2
        (KeyCodes::B, AlacModifiers::CtrlShift) => Some("\x02".to_string()), //2
        (KeyCodes::C, AlacModifiers::Ctrl) => Some("\x03".to_string()), //3
        (KeyCodes::C, AlacModifiers::CtrlShift) => Some("\x03".to_string()), //3
        (KeyCodes::D, AlacModifiers::Ctrl) => Some("\x04".to_string()), //4
        (KeyCodes::D, AlacModifiers::CtrlShift) => Some("\x04".to_string()), //4
        (KeyCodes::E, AlacModifiers::Ctrl) => Some("\x05".to_string()), //5
        (KeyCodes::E, AlacModifiers::CtrlShift) => Some("\x05".to_string()), //5
        (KeyCodes::F, AlacModifiers::Ctrl) => Some("\x06".to_string()), //6
        (KeyCodes::F, AlacModifiers::CtrlShift) => Some("\x06".to_string()), //6
        (KeyCodes::G, AlacModifiers::Ctrl) => Some("\x07".to_string()), //7
        (KeyCodes::G, AlacModifiers::CtrlShift) => Some("\x07".to_string()), //7
        (KeyCodes::H, AlacModifiers::Ctrl) => Some("\x08".to_string()), //8
        (KeyCodes::H, AlacModifiers::CtrlShift) => Some("\x08".to_string()), //8
        (KeyCodes::I, AlacModifiers::Ctrl) => Some("\x09".to_string()), //9
        (KeyCodes::I, AlacModifiers::CtrlShift) => Some("\x09".to_string()), //9
        (KeyCodes::J, AlacModifiers::Ctrl) => Some("\x0a".to_string()), //10
        (KeyCodes::J, AlacModifiers::CtrlShift) => Some("\x0a".to_string()), //10
        (KeyCodes::K, AlacModifiers::Ctrl) => Some("\x0b".to_string()), //11
        (KeyCodes::K, AlacModifiers::CtrlShift) => Some("\x0b".to_string()), //11
        (KeyCodes::L, AlacModifiers::Ctrl) => Some("\x0c".to_string()), //12
        (KeyCodes::L, AlacModifiers::CtrlShift) => Some("\x0c".to_string()), //12
        (KeyCodes::M, AlacModifiers::Ctrl) => Some("\x0d".to_string()), //13
        (KeyCodes::M, AlacModifiers::CtrlShift) => Some("\x0d".to_string()), //13
        (KeyCodes::N, AlacModifiers::Ctrl) => Some("\x0e".to_string()), //14
        (KeyCodes::N, AlacModifiers::CtrlShift) => Some("\x0e".to_string()), //14
        (KeyCodes::O, AlacModifiers::Ctrl) => Some("\x0f".to_string()), //15
        (KeyCodes::O, AlacModifiers::CtrlShift) => Some("\x0f".to_string()), //15
        (KeyCodes::P, AlacModifiers::Ctrl) => Some("\x10".to_string()), //16
        (KeyCodes::P, AlacModifiers::CtrlShift) => Some("\x10".to_string()), //16
        (KeyCodes::Q, AlacModifiers::Ctrl) => Some("\x11".to_string()), //17
        (KeyCodes::Q, AlacModifiers::CtrlShift) => Some("\x11".to_string()), //17
        (KeyCodes::R, AlacModifiers::Ctrl) => Some("\x12".to_string()), //18
        (KeyCodes::R, AlacModifiers::CtrlShift) => Some("\x12".to_string()), //18
        (KeyCodes::S, AlacModifiers::Ctrl) => Some("\x13".to_string()), //19
        (KeyCodes::S, AlacModifiers::CtrlShift) => Some("\x13".to_string()), //19
        (KeyCodes::T, AlacModifiers::Ctrl) => Some("\x14".to_string()), //20
        (KeyCodes::T, AlacModifiers::CtrlShift) => Some("\x14".to_string()), //20
        (KeyCodes::U, AlacModifiers::Ctrl) => Some("\x15".to_string()), //21
        (KeyCodes::U, AlacModifiers::CtrlShift) => Some("\x15".to_string()), //21
        (KeyCodes::V, AlacModifiers::Ctrl) => Some("\x16".to_string()), //22
        (KeyCodes::V, AlacModifiers::CtrlShift) => Some("\x16".to_string()), //22
        (KeyCodes::W, AlacModifiers::Ctrl) => Some("\x17".to_string()), //23
        (KeyCodes::W, AlacModifiers::CtrlShift) => Some("\x17".to_string()), //23
        (KeyCodes::X, AlacModifiers::Ctrl) => Some("\x18".to_string()), //24
        (KeyCodes::X, AlacModifiers::CtrlShift) => Some("\x18".to_string()), //24
        (KeyCodes::Y, AlacModifiers::Ctrl) => Some("\x19".to_string()), //25
        (KeyCodes::Y, AlacModifiers::CtrlShift) => Some("\x19".to_string()), //25
        (KeyCodes::Z, AlacModifiers::Ctrl) => Some("\x1a".to_string()), //26
        (KeyCodes::Z, AlacModifiers::CtrlShift) => Some("\x1a".to_string()), //26
        // TODO:
        // No @ key, just VirtualKeyCode::Digital2 + VirtualKeyCode::Shift
        // ("@", AlacModifiers::Ctrl) => Some("\x00".to_string()), //0
        (KeyCodes::LeftBracket, AlacModifiers::Ctrl) => Some("\x1b".to_string()), //27
        (KeyCodes::Backslash, AlacModifiers::Ctrl) => Some("\x1c".to_string()),   //28
        (KeyCodes::RightBracket, AlacModifiers::Ctrl) => Some("\x1d".to_string()), //29
        // TODO:
        // No ^ key, VirtualKeyCode::Digital6 + VirtualKeyCode::Shift
        // ("^", AlacModifiers::Ctrl) => Some("\x1e".to_string()), //30
        // TODO:
        // No _ key, VirtualKeyCode::OEMMinus + VirtualKeyCode::Shift
        // ("_", AlacModifiers::Ctrl) => Some("\x1f".to_string()), //31
        // TODO:
        // No ? key, VirtualKeyCode::OEM2 + VirtualKeyCode::Shift
        // ("?", AlacModifiers::Ctrl) => Some("\x7f".to_string()), //127
        _ => None,
    };
    if manual_esc_str.is_some() {
        return manual_esc_str;
    }

    // Automated bindings applying modifiers
    if modifiers.any() {
        let modifier_code = modifier_code(keystroke);
        let modified_esc_str = match keystroke.key {
            KeyCodes::Up => Some(format!("\x1b[1;{}A", modifier_code)),
            KeyCodes::Down => Some(format!("\x1b[1;{}B", modifier_code)),
            KeyCodes::Right => Some(format!("\x1b[1;{}C", modifier_code)),
            KeyCodes::Left => Some(format!("\x1b[1;{}D", modifier_code)),
            KeyCodes::F1 => Some(format!("\x1b[1;{}P", modifier_code)),
            KeyCodes::F2 => Some(format!("\x1b[1;{}Q", modifier_code)),
            KeyCodes::F3 => Some(format!("\x1b[1;{}R", modifier_code)),
            KeyCodes::F4 => Some(format!("\x1b[1;{}S", modifier_code)),
            KeyCodes::F5 => Some(format!("\x1b[15;{}~", modifier_code)),
            KeyCodes::F6 => Some(format!("\x1b[17;{}~", modifier_code)),
            KeyCodes::F7 => Some(format!("\x1b[18;{}~", modifier_code)),
            KeyCodes::F8 => Some(format!("\x1b[19;{}~", modifier_code)),
            KeyCodes::F9 => Some(format!("\x1b[20;{}~", modifier_code)),
            KeyCodes::F10 => Some(format!("\x1b[21;{}~", modifier_code)),
            KeyCodes::F11 => Some(format!("\x1b[23;{}~", modifier_code)),
            KeyCodes::F12 => Some(format!("\x1b[24;{}~", modifier_code)),
            KeyCodes::F13 => Some(format!("\x1b[25;{}~", modifier_code)),
            KeyCodes::F14 => Some(format!("\x1b[26;{}~", modifier_code)),
            KeyCodes::F15 => Some(format!("\x1b[28;{}~", modifier_code)),
            KeyCodes::F16 => Some(format!("\x1b[29;{}~", modifier_code)),
            KeyCodes::F17 => Some(format!("\x1b[31;{}~", modifier_code)),
            KeyCodes::F18 => Some(format!("\x1b[32;{}~", modifier_code)),
            KeyCodes::F19 => Some(format!("\x1b[33;{}~", modifier_code)),
            KeyCodes::F20 => Some(format!("\x1b[34;{}~", modifier_code)),
            _ if modifier_code == 2 => None,
            KeyCodes::Insert => Some(format!("\x1b[2;{}~", modifier_code)),
            KeyCodes::PageUp => Some(format!("\x1b[5;{}~", modifier_code)),
            KeyCodes::PageDown => Some(format!("\x1b[6;{}~", modifier_code)),
            KeyCodes::End => Some(format!("\x1b[1;{}F", modifier_code)),
            KeyCodes::Home => Some(format!("\x1b[1;{}H", modifier_code)),
            _ => None,
        };
        if modified_esc_str.is_some() {
            return modified_esc_str;
        }
    }

    let alt_meta_binding =
        // TODO:
        // if alt_is_meta && modifiers == AlacModifiers::Alt && keystroke.key.is_ascii() {
        if alt_is_meta && modifiers == AlacModifiers::Alt {
            // TODO:
            // Some(format!("\x1b{=}", keystroke.key))
            Some(format!("\x1b{:?}", keystroke.key))
        } else {
            None
        };

    if alt_meta_binding.is_some() {
        return alt_meta_binding;
    }

    None
}

///   Code     Modifiers
/// ---------+---------------------------
///    2     | Shift
///    3     | Alt
///    4     | Shift + Alt
///    5     | Control
///    6     | Shift + Control
///    7     | Alt + Control
///    8     | Shift + Alt + Control
/// ---------+---------------------------
/// from: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-PC-Style-Function-Keys
fn modifier_code(keystroke: &Keystroke) -> u32 {
    let mut modifier_code = 0;
    if keystroke.modifiers.shift {
        modifier_code |= 1;
    }
    if keystroke.modifiers.alt {
        modifier_code |= 1 << 1;
    }
    if keystroke.modifiers.control {
        modifier_code |= 1 << 2;
    }
    modifier_code + 1
}

#[cfg(test)]
mod test {
    // use gpui::Modifiers;

    use super::*;

    #[test]
    fn test_scroll_keys() {
        //These keys should be handled by the scrolling element directly
        //Need to signify this by returning 'None'
        let shift_pageup = Keystroke::parse("shift-pageup").unwrap();
        let shift_pagedown = Keystroke::parse("shift-pagedown").unwrap();
        let shift_home = Keystroke::parse("shift-home").unwrap();
        let shift_end = Keystroke::parse("shift-end").unwrap();

        let none = TermMode::NONE;
        assert_eq!(to_esc_str(&shift_pageup, &none, false), None);
        assert_eq!(to_esc_str(&shift_pagedown, &none, false), None);
        assert_eq!(to_esc_str(&shift_home, &none, false), None);
        assert_eq!(to_esc_str(&shift_end, &none, false), None);

        let alt_screen = TermMode::ALT_SCREEN;
        assert_eq!(
            to_esc_str(&shift_pageup, &alt_screen, false),
            Some("\x1b[5;2~".to_string())
        );
        assert_eq!(
            to_esc_str(&shift_pagedown, &alt_screen, false),
            Some("\x1b[6;2~".to_string())
        );
        assert_eq!(
            to_esc_str(&shift_home, &alt_screen, false),
            Some("\x1b[1;2H".to_string())
        );
        assert_eq!(
            to_esc_str(&shift_end, &alt_screen, false),
            Some("\x1b[1;2F".to_string())
        );

        let pageup = Keystroke::parse("pageup").unwrap();
        let pagedown = Keystroke::parse("pagedown").unwrap();
        let any = TermMode::ANY;

        assert_eq!(
            to_esc_str(&pageup, &any, false),
            Some("\x1b[5~".to_string())
        );
        assert_eq!(
            to_esc_str(&pagedown, &any, false),
            Some("\x1b[6~".to_string())
        );
    }

    // TODO:
    // Under VirtualKeyCode system, anthing that is considered "input", should go into
    // ime_key field.
    // #[test]
    // fn test_plain_inputs() {
    //     let ks = Keystroke {
    //         modifiers: Modifiers {
    //             control: false,
    //             alt: false,
    //             shift: false,
    //             platform: false,
    //             function: false,
    //         },
    //         key: "🖖🏻".to_string(), //2 char string
    //         ime_key: None,
    //     };
    //     assert_eq!(to_esc_str(&ks, &TermMode::NONE, false), None);
    // }

    #[test]
    fn test_application_mode() {
        let app_cursor = TermMode::APP_CURSOR;
        let none = TermMode::NONE;

        let up = Keystroke::parse("up").unwrap();
        let down = Keystroke::parse("down").unwrap();
        let left = Keystroke::parse("left").unwrap();
        let right = Keystroke::parse("right").unwrap();

        assert_eq!(to_esc_str(&up, &none, false), Some("\x1b[A".to_string()));
        assert_eq!(to_esc_str(&down, &none, false), Some("\x1b[B".to_string()));
        assert_eq!(to_esc_str(&right, &none, false), Some("\x1b[C".to_string()));
        assert_eq!(to_esc_str(&left, &none, false), Some("\x1b[D".to_string()));

        assert_eq!(
            to_esc_str(&up, &app_cursor, false),
            Some("\x1bOA".to_string())
        );
        assert_eq!(
            to_esc_str(&down, &app_cursor, false),
            Some("\x1bOB".to_string())
        );
        assert_eq!(
            to_esc_str(&right, &app_cursor, false),
            Some("\x1bOC".to_string())
        );
        assert_eq!(
            to_esc_str(&left, &app_cursor, false),
            Some("\x1bOD".to_string())
        );
    }

    #[test]
    fn test_ctrl_codes() {
        let letters_lower = 'a'..='z';
        let letters_upper = 'A'..='Z';
        let mode = TermMode::ANY;

        for (lower, upper) in letters_lower.zip(letters_upper) {
            assert_eq!(
                to_esc_str(
                    &Keystroke::parse(&format!("ctrl-{}", lower)).unwrap(),
                    &mode,
                    false
                ),
                to_esc_str(
                    &Keystroke::parse(&format!("ctrl-shift-{}", upper)).unwrap(),
                    &mode,
                    false
                ),
                "On letter: {}/{}",
                lower,
                upper
            )
        }
    }

    #[test]
    fn alt_is_meta() {
        let ascii_printable = ' '..='~';
        for character in ascii_printable {
            assert_eq!(
                to_esc_str(
                    &Keystroke::parse(&format!("alt-{}", character)).unwrap(),
                    &TermMode::NONE,
                    true
                )
                .unwrap(),
                format!("\x1b{}", character)
            );
        }

        let gpui_keys = [
            "up", "down", "right", "left", "f1", "f2", "f3", "f4", "F5", "f6", "f7", "f8", "f9",
            "f10", "f11", "f12", "f13", "f14", "f15", "f16", "f17", "f18", "f19", "f20", "insert",
            "pageup", "pagedown", "end", "home",
        ];

        for key in gpui_keys {
            assert_ne!(
                to_esc_str(
                    &Keystroke::parse(&format!("alt-{}", key)).unwrap(),
                    &TermMode::NONE,
                    true
                )
                .unwrap(),
                format!("\x1b{}", key)
            );
        }
    }

    #[test]
    fn test_modifier_code_calc() {
        //   Code     Modifiers
        // ---------+---------------------------
        //    2     | Shift
        //    3     | Alt
        //    4     | Shift + Alt
        //    5     | Control
        //    6     | Shift + Control
        //    7     | Alt + Control
        //    8     | Shift + Alt + Control
        // ---------+---------------------------
        // from: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-PC-Style-Function-Keys
        assert_eq!(2, modifier_code(&Keystroke::parse("shift-A").unwrap()));
        assert_eq!(3, modifier_code(&Keystroke::parse("alt-A").unwrap()));
        assert_eq!(4, modifier_code(&Keystroke::parse("shift-alt-A").unwrap()));
        assert_eq!(5, modifier_code(&Keystroke::parse("ctrl-A").unwrap()));
        assert_eq!(6, modifier_code(&Keystroke::parse("shift-ctrl-A").unwrap()));
        assert_eq!(7, modifier_code(&Keystroke::parse("alt-ctrl-A").unwrap()));
        assert_eq!(
            8,
            modifier_code(&Keystroke::parse("shift-ctrl-alt-A").unwrap())
        );
    }
}
