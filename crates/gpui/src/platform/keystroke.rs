use anyhow::anyhow;
use serde::Deserialize;
use std::fmt::Write;

use crate::keycodes::VirtualKeyCode;

/// A keystroke and associated metadata generated by the platform
#[derive(Clone, Debug, Eq, PartialEq, Default, Deserialize, Hash)]
pub struct Keystroke {
    /// the state of the modifier keys at the time the keystroke was generated
    pub modifiers: Modifiers,

    /// TODO:
    /// key is the character printed on the key that was pressed
    /// e.g. for option-s, key is "s"
    pub key: VirtualKeyCode,

    /// TODO: This is the key that use to print.
    /// ime_key is the character inserted by the IME engine when that key was pressed.
    /// e.g. for option-s, ime_key is "ß"
    pub ime_key: Option<String>,
}

impl Keystroke {
    /// When matching a key we cannot know whether the user intended to type
    /// the ime_key or the key itself. On some non-US keyboards keys we use in our
    /// bindings are behind option (for example `$` is typed `alt-ç` on a Czech keyboard),
    /// and on some keyboards the IME handler converts a sequence of keys into a
    /// specific character (for example `"` is typed as `" space` on a brazilian keyboard).
    ///
    /// This method assumes that `self` was typed and `target' is in the keymap, and checks
    /// both possibilities for self against the target.
    // pub(crate) fn should_match(&self, target: &Keystroke) -> bool {
    //     if let Some(ime_key) = self
    //         .ime_key
    //         .as_ref()
    //         .filter(|ime_key| ime_key != &&self.key)
    //     {
    //         let ime_modifiers = Modifiers {
    //             control: self.modifiers.control,
    //             ..Default::default()
    //         };

    //         if &target.key == ime_key && target.modifiers == ime_modifiers {
    //             return true;
    //         }
    //     }

    //     target.modifiers == self.modifiers && target.key == self.key
    // }

    /// key syntax is:
    /// [ctrl-][alt-][shift-][cmd-][fn-]key[->ime_key]
    /// ime_key syntax is only used for generating test events,
    /// when matching a key with an ime_key set will be matched without it.
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        let mut control = false;
        let mut alt = false;
        let mut shift = false;
        let mut platform = false;
        let mut function = false;
        let mut key = None;
        let mut ime_key = None;

        let mut components = source.split('-').peekable();
        while let Some(component) = components.next() {
            match component {
                "ctrl" => control = true,
                "alt" => alt = true,
                "shift" => shift = true,
                "fn" => function = true,
                "cmd" | "super" | "win" => platform = true,
                _ => {
                    if let Some(next) = components.peek() {
                        if next.is_empty() && source.ends_with('-') {
                            // key = Some(String::from("-"));
                            key = Some(VirtualKeyCode::from_str("-"));
                            break;
                        } else if next.len() > 1 && next.starts_with('>') {
                            // key = Some(String::from(component));
                            key = Some(VirtualKeyCode::from_str(component));
                            ime_key = Some(String::from(&next[1..]));
                            components.next();
                        } else {
                            return Err(anyhow!("Invalid keystroke `{}`", source));
                        }
                    } else {
                        // key = Some(String::from(component));
                        key = Some(VirtualKeyCode::from_str(component));
                    }
                }
            }
        }

        // TODO:
        //Allow for the user to specify a keystroke modifier as the key itself
        //This sets the `key` to the modifier, and disables the modifier
        if key.is_none() {
            if shift {
                // key = Some("shift".to_string());
                key = Some(VirtualKeyCode::Shift);
                shift = false;
            } else if control {
                // key = Some("control".to_string());
                key = Some(VirtualKeyCode::Control);
                control = false;
            } else if alt {
                // key = Some("alt".to_string());
                key = Some(VirtualKeyCode::Alt);
                alt = false;
            } else if platform {
                // key = Some("platform".to_string());
                key = Some(VirtualKeyCode::LeftPlatform);
                platform = false;
            } else if function {
                // key = Some("function".to_string());
                key = Some(VirtualKeyCode::Function);
                function = false;
            }
        }
        println!("      => key: {:?}", key);

        let key = key.ok_or_else(|| anyhow!("Invalid keystroke `{}`", source))?;

        Ok(Keystroke {
            modifiers: Modifiers {
                control,
                alt,
                shift,
                platform,
                function,
            },
            key,
            ime_key,
        })
    }

    // TODO: https://github.com/zed-industries/zed/pull/13185
    /// Returns true if this keystroke left
    /// the ime system in an incomplete state.
    // pub fn is_ime_in_progress(&self) -> bool {
    //     self.ime_key.is_none()
    //         && (is_printable_key(&self.key) || self.key.is_empty())
    //         && !(self.modifiers.platform
    //             || self.modifiers.control
    //             || self.modifiers.function
    //             || self.modifiers.alt)
    // }

    /// Returns a new keystroke with the ime_key filled.
    /// This is used for dispatch_keystroke where we want users to
    /// be able to simulate typing "space", etc.
    pub fn with_simulated_ime(mut self) -> Self {
        if self.ime_key.is_none()
            && !self.modifiers.platform
            && !self.modifiers.control
            && !self.modifiers.function
            && !self.modifiers.alt
        {
            self.ime_key = match self.key {
                // "space" => Some(" ".into()),
                // "tab" => Some("\t".into()),
                // "enter" => Some("\n".into()),
                // key if !is_printable_key(key) => None,
                // key => {
                //     if self.modifiers.shift {
                //         Some(key.to_uppercase())
                //     } else {
                //         Some(key.into())
                //     }
                // }
                VirtualKeyCode::Space => Some(" ".into()),
                VirtualKeyCode::Tab => Some("\t".into()),
                VirtualKeyCode::Enter => Some("\n".into()),
                // TODO:
                key if !key.is_printable() => None,
                key => {
                    if self.modifiers.shift {
                        Some(key.to_string().to_uppercase())
                    } else {
                        Some(key.to_string())
                    }
                }
            }
        }
        self
    }
}

fn is_printable_key(key: &str) -> bool {
    !matches!(
        key,
        "f1" | "f2"
            | "f3"
            | "f4"
            | "f5"
            | "f6"
            | "f7"
            | "f8"
            | "f9"
            | "f10"
            | "f11"
            | "f12"
            | "f13"
            | "f14"
            | "f15"
            | "f16"
            | "f17"
            | "f18"
            | "f19"
            | "backspace"
            | "delete"
            | "left"
            | "right"
            | "up"
            | "down"
            | "pageup"
            | "pagedown"
            | "insert"
            | "home"
            | "end"
            | "escape"
    )
}

impl std::fmt::Display for Keystroke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.control {
            f.write_char('^')?;
        }
        if self.modifiers.alt {
            f.write_char('⌥')?;
        }
        if self.modifiers.platform {
            #[cfg(target_os = "macos")]
            f.write_char('⌘')?;

            #[cfg(target_os = "linux")]
            f.write_char('❖')?;

            #[cfg(target_os = "windows")]
            f.write_char('⊞')?;
        }
        if self.modifiers.shift {
            f.write_char('⇧')?;
        }
        // let key = match self.key.as_str() {
        //     "backspace" => '⌫',
        //     "up" => '↑',
        //     "down" => '↓',
        //     "left" => '←',
        //     "right" => '→',
        //     "tab" => '⇥',
        //     "escape" => '⎋',
        //     "shift" => '⇧',
        //     "control" => '⌃',
        //     "alt" => '⌥',
        //     "platform" => '⌘',
        //     key => {
        //         if key.len() == 1 {
        //             key.chars().next().unwrap().to_ascii_uppercase()
        //         } else {
        //             return f.write_str(key);
        //         }
        //     }
        // };
        let key = match self.key {
            VirtualKeyCode::Backspace => '⌫',
            VirtualKeyCode::Up => '↑',
            VirtualKeyCode::Down => '↓',
            VirtualKeyCode::Left => '←',
            VirtualKeyCode::Right => '→',
            VirtualKeyCode::Tab => '⇥',
            VirtualKeyCode::Escape => '⎋',
            VirtualKeyCode::Shift => '⇧',
            VirtualKeyCode::Control => '⌃',
            VirtualKeyCode::Alt => '⌥',
            VirtualKeyCode::LeftPlatform => '⌘',
            key => {
                let key = key.to_string();
                if key.len() == 1 {
                    key.chars().next().unwrap().to_ascii_uppercase()
                } else {
                    return f.write_str(&key);
                }
            }
        };
        f.write_char(key)
    }
}

/// The state of the modifier keys at some point in time
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Deserialize, Hash)]
pub struct Modifiers {
    /// The control key
    pub control: bool,

    /// The alt key
    /// Sometimes also known as the 'meta' key
    pub alt: bool,

    /// The shift key
    pub shift: bool,

    /// The command key, on macos
    /// the windows key, on windows
    /// the super key, on linux
    pub platform: bool,

    /// The function key
    pub function: bool,
}

impl Modifiers {
    /// Returns whether any modifier key is pressed.
    pub fn modified(&self) -> bool {
        self.control || self.alt || self.shift || self.platform || self.function
    }

    /// Whether the semantically 'secondary' modifier key is pressed.
    ///
    /// On macOS, this is the command key.
    /// On Linux and Windows, this is the control key.
    pub fn secondary(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.platform
        }

        #[cfg(not(target_os = "macos"))]
        {
            self.control
        }
    }

    /// Returns how many modifier keys are pressed.
    pub fn number_of_modifiers(&self) -> u8 {
        self.control as u8
            + self.alt as u8
            + self.shift as u8
            + self.platform as u8
            + self.function as u8
    }

    /// Returns [`Modifiers`] with no modifiers.
    pub fn none() -> Modifiers {
        Default::default()
    }

    /// Returns [`Modifiers`] with just the command key.
    pub fn command() -> Modifiers {
        Modifiers {
            platform: true,
            ..Default::default()
        }
    }

    /// A Returns [`Modifiers`] with just the secondary key pressed.
    pub fn secondary_key() -> Modifiers {
        #[cfg(target_os = "macos")]
        {
            Modifiers {
                platform: true,
                ..Default::default()
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Modifiers {
                control: true,
                ..Default::default()
            }
        }
    }

    /// Returns [`Modifiers`] with just the windows key.
    pub fn windows() -> Modifiers {
        Modifiers {
            platform: true,
            ..Default::default()
        }
    }

    /// Returns [`Modifiers`] with just the super key.
    pub fn super_key() -> Modifiers {
        Modifiers {
            platform: true,
            ..Default::default()
        }
    }

    /// Returns [`Modifiers`] with just control.
    pub fn control() -> Modifiers {
        Modifiers {
            control: true,
            ..Default::default()
        }
    }

    /// Returns [`Modifiers`] with just control.
    pub fn alt() -> Modifiers {
        Modifiers {
            alt: true,
            ..Default::default()
        }
    }

    /// Returns [`Modifiers`] with just shift.
    pub fn shift() -> Modifiers {
        Modifiers {
            shift: true,
            ..Default::default()
        }
    }

    /// Returns [`Modifiers`] with command + shift.
    pub fn command_shift() -> Modifiers {
        Modifiers {
            shift: true,
            platform: true,
            ..Default::default()
        }
    }

    /// Returns [`Modifiers`] with command + shift.
    pub fn control_shift() -> Modifiers {
        Modifiers {
            shift: true,
            control: true,
            ..Default::default()
        }
    }

    /// Checks if this [`Modifiers`] is a subset of another [`Modifiers`].
    pub fn is_subset_of(&self, other: &Modifiers) -> bool {
        (other.control || !self.control)
            && (other.alt || !self.alt)
            && (other.shift || !self.shift)
            && (other.platform || !self.platform)
            && (other.function || !self.function)
    }
}

#[cfg(test)]
mod tests {
    use crate::Keystroke;

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
        assert_eq!(2, modifier_code(&Keystroke::parse("shift-a").unwrap()));
        assert_eq!(3, modifier_code(&Keystroke::parse("alt-a").unwrap()));
        assert_eq!(4, modifier_code(&Keystroke::parse("shift-alt-a").unwrap()));
        assert_eq!(5, modifier_code(&Keystroke::parse("ctrl-a").unwrap()));
        assert_eq!(6, modifier_code(&Keystroke::parse("shift-ctrl-a").unwrap()));
        assert_eq!(7, modifier_code(&Keystroke::parse("alt-ctrl-a").unwrap()));
        assert_eq!(
            8,
            modifier_code(&Keystroke::parse("shift-ctrl-alt-a").unwrap())
        );
    }
}
