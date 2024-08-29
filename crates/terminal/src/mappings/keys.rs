/// The mappings defined in this file where created from reading the alacritty source
use alacritty_terminal::term::TermMode;
use gpui::{Keystroke, VirtualKeyCode};

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
        (VirtualKeyCode::Tab, AlacModifiers::None) => Some("\x09".to_string()),
        (VirtualKeyCode::Escape, AlacModifiers::None) => Some("\x1b".to_string()),
        (VirtualKeyCode::Enter, AlacModifiers::None) => Some("\x0d".to_string()),
        (VirtualKeyCode::Enter, AlacModifiers::Shift) => Some("\x0d".to_string()),
        (VirtualKeyCode::Backspace, AlacModifiers::None) => Some("\x7f".to_string()),
        //Interesting escape codes
        (VirtualKeyCode::Tab, AlacModifiers::Shift) => Some("\x1b[Z".to_string()),
        (VirtualKeyCode::Backspace, AlacModifiers::Ctrl) => Some("\x08".to_string()),
        (VirtualKeyCode::Backspace, AlacModifiers::Alt) => Some("\x1b\x7f".to_string()),
        (VirtualKeyCode::Backspace, AlacModifiers::Shift) => Some("\x7f".to_string()),
        (VirtualKeyCode::Space, AlacModifiers::Ctrl) => Some("\x00".to_string()),
        (VirtualKeyCode::Home, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[1;2H".to_string())
        }
        (VirtualKeyCode::End, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[1;2F".to_string())
        }
        (VirtualKeyCode::PageUp, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[5;2~".to_string())
        }
        (VirtualKeyCode::PageDown, AlacModifiers::Shift) if mode.contains(TermMode::ALT_SCREEN) => {
            Some("\x1b[6;2~".to_string())
        }
        (VirtualKeyCode::Home, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOH".to_string())
        }
        (VirtualKeyCode::Home, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[H".to_string())
        }
        (VirtualKeyCode::End, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOF".to_string())
        }
        (VirtualKeyCode::End, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[F".to_string())
        }
        (VirtualKeyCode::Up, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOA".to_string())
        }
        (VirtualKeyCode::Up, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[A".to_string())
        }
        (VirtualKeyCode::Down, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOB".to_string())
        }
        (VirtualKeyCode::Down, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[B".to_string())
        }
        (VirtualKeyCode::Right, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOC".to_string())
        }
        (VirtualKeyCode::Right, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[C".to_string())
        }
        (VirtualKeyCode::Left, AlacModifiers::None) if mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1bOD".to_string())
        }
        (VirtualKeyCode::Left, AlacModifiers::None) if !mode.contains(TermMode::APP_CURSOR) => {
            Some("\x1b[D".to_string())
        }
        // TODO: correct?
        // ("back", AlacModifiers::None) => Some("\x7f".to_string()),
        (VirtualKeyCode::Backspace, AlacModifiers::None) => Some("\x7f".to_string()),
        (VirtualKeyCode::Insert, AlacModifiers::None) => Some("\x1b[2~".to_string()),
        (VirtualKeyCode::Delete, AlacModifiers::None) => Some("\x1b[3~".to_string()),
        (VirtualKeyCode::PageUp, AlacModifiers::None) => Some("\x1b[5~".to_string()),
        (VirtualKeyCode::PageDown, AlacModifiers::None) => Some("\x1b[6~".to_string()),
        (VirtualKeyCode::F1, AlacModifiers::None) => Some("\x1bOP".to_string()),
        (VirtualKeyCode::F2, AlacModifiers::None) => Some("\x1bOQ".to_string()),
        (VirtualKeyCode::F3, AlacModifiers::None) => Some("\x1bOR".to_string()),
        (VirtualKeyCode::F4, AlacModifiers::None) => Some("\x1bOS".to_string()),
        (VirtualKeyCode::F5, AlacModifiers::None) => Some("\x1b[15~".to_string()),
        (VirtualKeyCode::F6, AlacModifiers::None) => Some("\x1b[17~".to_string()),
        (VirtualKeyCode::F7, AlacModifiers::None) => Some("\x1b[18~".to_string()),
        (VirtualKeyCode::F8, AlacModifiers::None) => Some("\x1b[19~".to_string()),
        (VirtualKeyCode::F9, AlacModifiers::None) => Some("\x1b[20~".to_string()),
        (VirtualKeyCode::F10, AlacModifiers::None) => Some("\x1b[21~".to_string()),
        (VirtualKeyCode::F11, AlacModifiers::None) => Some("\x1b[23~".to_string()),
        (VirtualKeyCode::F12, AlacModifiers::None) => Some("\x1b[24~".to_string()),
        (VirtualKeyCode::F13, AlacModifiers::None) => Some("\x1b[25~".to_string()),
        (VirtualKeyCode::F14, AlacModifiers::None) => Some("\x1b[26~".to_string()),
        (VirtualKeyCode::F15, AlacModifiers::None) => Some("\x1b[28~".to_string()),
        (VirtualKeyCode::F16, AlacModifiers::None) => Some("\x1b[29~".to_string()),
        (VirtualKeyCode::F17, AlacModifiers::None) => Some("\x1b[31~".to_string()),
        (VirtualKeyCode::F18, AlacModifiers::None) => Some("\x1b[32~".to_string()),
        (VirtualKeyCode::F19, AlacModifiers::None) => Some("\x1b[33~".to_string()),
        (VirtualKeyCode::F20, AlacModifiers::None) => Some("\x1b[34~".to_string()),
        // NumpadEnter, Action::Esc("\n".into());
        //Mappings for caret notation keys
        // TODO:
        (VirtualKeyCode::A, AlacModifiers::Ctrl) => Some("\x01".to_string()), //1
        (VirtualKeyCode::A, AlacModifiers::CtrlShift) => Some("\x01".to_string()), //1
        (VirtualKeyCode::B, AlacModifiers::Ctrl) => Some("\x02".to_string()), //2
        (VirtualKeyCode::B, AlacModifiers::CtrlShift) => Some("\x02".to_string()), //2
        (VirtualKeyCode::C, AlacModifiers::Ctrl) => Some("\x03".to_string()), //3
        (VirtualKeyCode::C, AlacModifiers::CtrlShift) => Some("\x03".to_string()), //3
        (VirtualKeyCode::D, AlacModifiers::Ctrl) => Some("\x04".to_string()), //4
        (VirtualKeyCode::D, AlacModifiers::CtrlShift) => Some("\x04".to_string()), //4
        (VirtualKeyCode::E, AlacModifiers::Ctrl) => Some("\x05".to_string()), //5
        (VirtualKeyCode::E, AlacModifiers::CtrlShift) => Some("\x05".to_string()), //5
        (VirtualKeyCode::F, AlacModifiers::Ctrl) => Some("\x06".to_string()), //6
        (VirtualKeyCode::F, AlacModifiers::CtrlShift) => Some("\x06".to_string()), //6
        (VirtualKeyCode::G, AlacModifiers::Ctrl) => Some("\x07".to_string()), //7
        (VirtualKeyCode::G, AlacModifiers::CtrlShift) => Some("\x07".to_string()), //7
        (VirtualKeyCode::H, AlacModifiers::Ctrl) => Some("\x08".to_string()), //8
        (VirtualKeyCode::H, AlacModifiers::CtrlShift) => Some("\x08".to_string()), //8
        (VirtualKeyCode::I, AlacModifiers::Ctrl) => Some("\x09".to_string()), //9
        (VirtualKeyCode::I, AlacModifiers::CtrlShift) => Some("\x09".to_string()), //9
        (VirtualKeyCode::J, AlacModifiers::Ctrl) => Some("\x0a".to_string()), //10
        (VirtualKeyCode::J, AlacModifiers::CtrlShift) => Some("\x0a".to_string()), //10
        (VirtualKeyCode::K, AlacModifiers::Ctrl) => Some("\x0b".to_string()), //11
        (VirtualKeyCode::K, AlacModifiers::CtrlShift) => Some("\x0b".to_string()), //11
        (VirtualKeyCode::L, AlacModifiers::Ctrl) => Some("\x0c".to_string()), //12
        (VirtualKeyCode::L, AlacModifiers::CtrlShift) => Some("\x0c".to_string()), //12
        (VirtualKeyCode::M, AlacModifiers::Ctrl) => Some("\x0d".to_string()), //13
        (VirtualKeyCode::M, AlacModifiers::CtrlShift) => Some("\x0d".to_string()), //13
        (VirtualKeyCode::N, AlacModifiers::Ctrl) => Some("\x0e".to_string()), //14
        (VirtualKeyCode::N, AlacModifiers::CtrlShift) => Some("\x0e".to_string()), //14
        (VirtualKeyCode::O, AlacModifiers::Ctrl) => Some("\x0f".to_string()), //15
        (VirtualKeyCode::O, AlacModifiers::CtrlShift) => Some("\x0f".to_string()), //15
        (VirtualKeyCode::P, AlacModifiers::Ctrl) => Some("\x10".to_string()), //16
        (VirtualKeyCode::P, AlacModifiers::CtrlShift) => Some("\x10".to_string()), //16
        (VirtualKeyCode::Q, AlacModifiers::Ctrl) => Some("\x11".to_string()), //17
        (VirtualKeyCode::Q, AlacModifiers::CtrlShift) => Some("\x11".to_string()), //17
        (VirtualKeyCode::R, AlacModifiers::Ctrl) => Some("\x12".to_string()), //18
        (VirtualKeyCode::R, AlacModifiers::CtrlShift) => Some("\x12".to_string()), //18
        (VirtualKeyCode::S, AlacModifiers::Ctrl) => Some("\x13".to_string()), //19
        (VirtualKeyCode::S, AlacModifiers::CtrlShift) => Some("\x13".to_string()), //19
        (VirtualKeyCode::T, AlacModifiers::Ctrl) => Some("\x14".to_string()), //20
        (VirtualKeyCode::T, AlacModifiers::CtrlShift) => Some("\x14".to_string()), //20
        (VirtualKeyCode::U, AlacModifiers::Ctrl) => Some("\x15".to_string()), //21
        (VirtualKeyCode::U, AlacModifiers::CtrlShift) => Some("\x15".to_string()), //21
        (VirtualKeyCode::V, AlacModifiers::Ctrl) => Some("\x16".to_string()), //22
        (VirtualKeyCode::V, AlacModifiers::CtrlShift) => Some("\x16".to_string()), //22
        (VirtualKeyCode::W, AlacModifiers::Ctrl) => Some("\x17".to_string()), //23
        (VirtualKeyCode::W, AlacModifiers::CtrlShift) => Some("\x17".to_string()), //23
        (VirtualKeyCode::X, AlacModifiers::Ctrl) => Some("\x18".to_string()), //24
        (VirtualKeyCode::X, AlacModifiers::CtrlShift) => Some("\x18".to_string()), //24
        (VirtualKeyCode::Y, AlacModifiers::Ctrl) => Some("\x19".to_string()), //25
        (VirtualKeyCode::Y, AlacModifiers::CtrlShift) => Some("\x19".to_string()), //25
        (VirtualKeyCode::Z, AlacModifiers::Ctrl) => Some("\x1a".to_string()), //26
        (VirtualKeyCode::Z, AlacModifiers::CtrlShift) => Some("\x1a".to_string()), //26
        // TODO:
        // No @ key, just VirtualKeyCode::Digital2 + VirtualKeyCode::Shift
        // ("@", AlacModifiers::Ctrl) => Some("\x00".to_string()), //0
        (VirtualKeyCode::OEM4, AlacModifiers::Ctrl) => Some("\x1b".to_string()), //27
        (VirtualKeyCode::OEM5, AlacModifiers::Ctrl) => Some("\x1c".to_string()), //28
        (VirtualKeyCode::OEM6, AlacModifiers::Ctrl) => Some("\x1d".to_string()), //29
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
            VirtualKeyCode::Up => Some(format!("\x1b[1;{}A", modifier_code)),
            VirtualKeyCode::Down => Some(format!("\x1b[1;{}B", modifier_code)),
            VirtualKeyCode::Right => Some(format!("\x1b[1;{}C", modifier_code)),
            VirtualKeyCode::Left => Some(format!("\x1b[1;{}D", modifier_code)),
            VirtualKeyCode::F1 => Some(format!("\x1b[1;{}P", modifier_code)),
            VirtualKeyCode::F2 => Some(format!("\x1b[1;{}Q", modifier_code)),
            VirtualKeyCode::F3 => Some(format!("\x1b[1;{}R", modifier_code)),
            VirtualKeyCode::F4 => Some(format!("\x1b[1;{}S", modifier_code)),
            VirtualKeyCode::F5 => Some(format!("\x1b[15;{}~", modifier_code)),
            VirtualKeyCode::F6 => Some(format!("\x1b[17;{}~", modifier_code)),
            VirtualKeyCode::F7 => Some(format!("\x1b[18;{}~", modifier_code)),
            VirtualKeyCode::F8 => Some(format!("\x1b[19;{}~", modifier_code)),
            VirtualKeyCode::F9 => Some(format!("\x1b[20;{}~", modifier_code)),
            VirtualKeyCode::F10 => Some(format!("\x1b[21;{}~", modifier_code)),
            VirtualKeyCode::F11 => Some(format!("\x1b[23;{}~", modifier_code)),
            VirtualKeyCode::F12 => Some(format!("\x1b[24;{}~", modifier_code)),
            VirtualKeyCode::F13 => Some(format!("\x1b[25;{}~", modifier_code)),
            VirtualKeyCode::F14 => Some(format!("\x1b[26;{}~", modifier_code)),
            VirtualKeyCode::F15 => Some(format!("\x1b[28;{}~", modifier_code)),
            VirtualKeyCode::F16 => Some(format!("\x1b[29;{}~", modifier_code)),
            VirtualKeyCode::F17 => Some(format!("\x1b[31;{}~", modifier_code)),
            VirtualKeyCode::F18 => Some(format!("\x1b[32;{}~", modifier_code)),
            VirtualKeyCode::F19 => Some(format!("\x1b[33;{}~", modifier_code)),
            VirtualKeyCode::F20 => Some(format!("\x1b[34;{}~", modifier_code)),
            _ if modifier_code == 2 => None,
            VirtualKeyCode::Insert => Some(format!("\x1b[2;{}~", modifier_code)),
            VirtualKeyCode::PageUp => Some(format!("\x1b[5;{}~", modifier_code)),
            VirtualKeyCode::PageDown => Some(format!("\x1b[6;{}~", modifier_code)),
            VirtualKeyCode::End => Some(format!("\x1b[1;{}F", modifier_code)),
            VirtualKeyCode::Home => Some(format!("\x1b[1;{}H", modifier_code)),
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
