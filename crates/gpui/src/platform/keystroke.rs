use anyhow::anyhow;
use serde::Deserialize;
use std::fmt::Write;
use util::ResultExt;

use crate::{keycodes::KeyCodes, KeyPosition};

/// A keystroke and associated metadata generated by the platform
#[derive(Clone, Debug, Eq, PartialEq, Default, Deserialize, Hash)]
pub struct Keystroke {
    /// the state of the modifier keys at the time the keystroke was generated
    pub modifiers: Modifiers,

    /// TODO:
    /// key is the character printed on the key that was pressed
    /// e.g. for option-s, key is "s"
    pub key: KeyCodes,

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
    // TODO:
    // Is the hack above still needed?
    pub(crate) fn should_match(&self, target: &Keystroke) -> bool {
        // if let Some(ime_key) = self
        //     .ime_key
        //     .as_ref()
        //     .filter(|ime_key| ime_key != &&self.key)
        // {
        //     let ime_modifiers = Modifiers {
        //         control: self.modifiers.control,
        //         ..Default::default()
        //     };

        //     if &target.key == ime_key && target.modifiers == ime_modifiers {
        //         return true;
        //     }
        // }

        target.modifiers == self.modifiers && target.key == self.key
    }

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
                            key = Some(KeyCodes::from_str("-"));
                            break;
                        } else if next.len() > 1 && next.starts_with('>') {
                            key = Some(KeyCodes::from_str(component));
                            ime_key = Some(String::from(&next[1..]));
                            components.next();
                        } else {
                            return Err(anyhow!("Invalid keystroke `{}`", source));
                        }
                    } else if let Some(translated) = translate_capital_keystroke(component) {
                        if shift {
                            log::error!(
                                "Error parsing keystroke `{}`, double shift detected.",
                                source
                            );
                        }
                        shift = true;
                        key = Some(KeyCodes::from_str(&translated));
                    } else {
                        key = Some(KeyCodes::from_str(component));
                    }
                }
            }
        }

        //Allow for the user to specify a keystroke modifier as the key itself
        //This sets the `key` to the modifier, and disables the modifier
        if key.is_none() {
            if shift {
                key = Some(Ok(KeyCodes::Shift(KeyPosition::Any)));
                shift = false;
            } else if control {
                key = Some(Ok(KeyCodes::Control(KeyPosition::Any)));
                control = false;
            } else if alt {
                key = Some(Ok(KeyCodes::Alt(KeyPosition::Any)));
                alt = false;
            } else if platform {
                key = Some(Ok(KeyCodes::Platform(KeyPosition::Any)));
                platform = false;
            } else if function {
                key = Some(Ok(KeyCodes::Function));
                function = false;
            }
        }

        // TODO:
        // Return Err here actually makes the app panicing.
        let key = key
            .ok_or_else(|| anyhow!("Invalid keystroke `{}`", source))?
            .log_err()
            .unwrap_or_default();

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
    pub fn is_ime_in_progress(&self) -> bool {
        self.ime_key.is_none()
            && (self.key.is_printable() || self.key == KeyCodes::Unknown)
            && !(self.modifiers.platform
                || self.modifiers.control
                || self.modifiers.function
                || self.modifiers.alt)
    }

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
                KeyCodes::Space => Some(" ".into()),
                KeyCodes::Tab => Some("\t".into()),
                KeyCodes::Enter => Some("\n".into()),
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
            KeyCodes::Backspace => '⌫',
            KeyCodes::Up => '↑',
            KeyCodes::Down => '↓',
            KeyCodes::Left => '←',
            KeyCodes::Right => '→',
            KeyCodes::Tab => '⇥',
            KeyCodes::Escape => '⎋',
            KeyCodes::Shift(_) => '⇧',
            KeyCodes::Control(_) => '⌃',
            KeyCodes::Alt(_) => '⌥',
            KeyCodes::Platform(_) => '⌘',
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

fn translate_capital_keystroke(input: &str) -> Option<String> {
    if input.len() != 1 {
        return None;
    }
    let ch = input.chars().next().unwrap();
    if ch.is_ascii_alphabetic() {
        if ch.is_ascii_uppercase() {
            return Some(input.to_lowercase());
        } else {
            return None;
        }
    }
    match ch {
        '~' => Some("`".to_string()),
        '!' => Some("1".to_string()),
        '@' => Some("2".to_string()),
        '#' => Some("3".to_string()),
        '$' => Some("4".to_string()),
        '%' => Some("5".to_string()),
        '^' => Some("6".to_string()),
        '&' => Some("7".to_string()),
        '*' => Some("8".to_string()),
        '(' => Some("9".to_string()),
        ')' => Some("0".to_string()),
        '_' => Some("-".to_string()),
        '+' => Some("=".to_string()),
        '{' => Some("[".to_string()),
        '}' => Some("]".to_string()),
        '|' => Some("\\".to_string()),
        ':' => Some(";".to_string()),
        '"' => Some("'".to_string()),
        '<' => Some(",".to_string()),
        '>' => Some(".".to_string()),
        '?' => Some("/".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{KeyCodes, Keystroke, Modifiers};

    // TODO:
    // Add tests for different keyboard layouts
    #[test]
    fn test_keystroke_parse() {
        // | Keystrokes
        // +---------------------------
        // | shift
        // | alt
        // | shift-a
        // | shift-,
        // | alt-q
        // | cmd-shift-[
        // | ctrl-shift-\
        // | a
        // +---------------------------
        assert_eq!(
            Keystroke {
                key: KeyCodes::Shift(crate::KeyPosition::Any),
                ..Default::default()
            },
            Keystroke::parse("shift").unwrap()
        );
        assert_eq!(
            Keystroke {
                key: KeyCodes::Alt(crate::KeyPosition::Any),
                ..Default::default()
            },
            Keystroke::parse("alt").unwrap()
        );
        assert_eq!(
            Keystroke {
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                key: KeyCodes::A,
                ..Default::default()
            },
            Keystroke::parse("shift-a").unwrap()
        );
        assert_eq!(
            Keystroke {
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                key: KeyCodes::Comma,
                ..Default::default()
            },
            Keystroke::parse("shift-,").unwrap()
        );
        assert_eq!(
            Keystroke {
                modifiers: Modifiers {
                    alt: true,
                    ..Default::default()
                },
                key: KeyCodes::Q,
                ..Default::default()
            },
            Keystroke::parse("alt-q").unwrap()
        );
        #[cfg(target_os = "macos")]
        assert_eq!(
            Keystroke {
                modifiers: Modifiers {
                    platform: true,
                    shift: true,
                    ..Default::default()
                },
                key: KeyCodes::LeftBracket,
                ..Default::default()
            },
            Keystroke::parse("cmd-shift-[").unwrap()
        );
        assert_eq!(
            Keystroke {
                modifiers: Modifiers {
                    control: true,
                    shift: true,
                    ..Default::default()
                },
                key: KeyCodes::Backslash,
                ..Default::default()
            },
            Keystroke::parse("ctrl-shift-\\").unwrap()
        );
        assert_eq!(
            Keystroke {
                key: KeyCodes::A,
                ..Default::default()
            },
            Keystroke::parse("a").unwrap()
        );
    }
}
