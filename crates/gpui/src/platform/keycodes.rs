/// On Windows, this is the Virtual-Key Codes
/// https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
/// On macOS and Linux, this is the Scan Codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum KeyCode {
    /// Un-recognized key
    #[default]
    Unknown,
    /// Fn on macOS
    Function,
    /// Control-break processing, `VK_CANCEL` on Windows.
    Cancel,
    /// BACKSPACE key, `VK_BACK` on Windows.
    Backspace,
    /// TAB key, `VK_TAB` on Windows.
    Tab,
    /// CLEAR key, `VK_CLEAR` on Windows.
    Clear,
    /// RETURN key, `VK_RETURN` on Windows.
    Enter,
    /// SHIFT key, `VK_SHIFT` on Windows. Note, both left-shift and right-shift can
    /// trigger this.
    Shift(KeyPosition),
    /// CTRL key, `VK_CONTROL` on Windows. Note, both left-ctrl and right-ctrl can
    /// trigger this.
    Control(KeyPosition),
    /// Alt key, `VK_MENU` on Windows. Note, both left-alt and right-alt can
    /// trigger this.
    Alt(KeyPosition),
    /// PAUSE key, `VK_PAUSE` on Windows.
    Pause,
    /// CAPS LOCK key, `VK_CAPITAL` on Windows.
    Capital,
    /// ESC key, `VK_ESCAPE` on Windows.
    Escape,
    /// SPACEBAR, `VK_SPACE` on Windows.
    Space,
    /// PAGE UP key, `VK_PRIOR` on Windows.
    PageUp,
    /// PAGE DOWN key, `VK_NEXT` on Windows.
    PageDown,
    /// END key, `VK_END` on Windows.
    End,
    /// HOME key, `VK_HOME` on Windows.
    Home,
    /// LEFT ARROW key, `VK_LEFT` on Windows.
    Left,
    /// UP ARROW key, `VK_UP` on Windows.
    Up,
    /// RIGHT ARROW key, `VK_RIGHT` on Winodws.
    Right,
    /// DOWN ARROW key, `VK_DOWN` on Windows.
    Down,
    /// SELECT key, `VK_SELECT` on Winodws.
    Select,
    /// PRINT key, `VK_PRINT` on Windows.
    Print,
    /// PRINT SCREEN key, `VK_SNAPSHOT` on Windows.
    PrintScreen,
    /// INS key, `VK_INSERT` on Windows.
    Insert,
    /// DEL key, `VK_DELETE` on Windows.
    Delete,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital0,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital1,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital2,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital3,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital4,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital5,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital6,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital7,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital8,
    /// 0 key on the main keyboard, `VK_0` on Windows.
    Digital9,
    /// A key on the main keyboard, `VK_A` on Windows.
    A,
    /// A key on the main keyboard, `VK_A` on Windows.
    B,
    /// A key on the main keyboard, `VK_A` on Windows.
    C,
    /// A key on the main keyboard, `VK_A` on Windows.
    D,
    /// A key on the main keyboard, `VK_A` on Windows.
    E,
    /// A key on the main keyboard, `VK_A` on Windows.
    F,
    /// A key on the main keyboard, `VK_A` on Windows.
    G,
    /// A key on the main keyboard, `VK_A` on Windows.
    H,
    /// A key on the main keyboard, `VK_A` on Windows.
    I,
    /// A key on the main keyboard, `VK_A` on Windows.
    J,
    /// A key on the main keyboard, `VK_A` on Windows.
    K,
    /// A key on the main keyboard, `VK_A` on Windows.
    L,
    /// A key on the main keyboard, `VK_A` on Windows.
    M,
    /// A key on the main keyboard, `VK_A` on Windows.
    N,
    /// A key on the main keyboard, `VK_A` on Windows.
    O,
    /// A key on the main keyboard, `VK_A` on Windows.
    P,
    /// A key on the main keyboard, `VK_A` on Windows.
    Q,
    /// A key on the main keyboard, `VK_A` on Windows.
    R,
    /// A key on the main keyboard, `VK_A` on Windows.
    S,
    /// A key on the main keyboard, `VK_A` on Windows.
    T,
    /// A key on the main keyboard, `VK_A` on Windows.
    U,
    /// A key on the main keyboard, `VK_A` on Windows.
    V,
    /// A key on the main keyboard, `VK_A` on Windows.
    W,
    /// A key on the main keyboard, `VK_A` on Windows.
    X,
    /// A key on the main keyboard, `VK_A` on Windows.
    Y,
    /// A key on the main keyboard, `VK_A` on Windows.
    Z,
    /// WIN key
    Platform(KeyPosition),
    /// Applications key, `VK_APPS` on Windows.
    App,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad0,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad1,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad2,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad3,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad4,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad5,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad6,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad7,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad8,
    // /// Numeric keypad 0 key, `VK_NUMPAD0` on Windows.
    // Numpad9,
    // /// Multiply key, `VK_MULTIPLY` on Windows.
    // Multiply,
    // /// Add key, `VK_ADD` on Windows.
    // Add,
    // /// Separator key, `VK_SEPARATOR` on Windows.
    // Separator,
    // /// Subtract key, `VK_SUBTRACT` on Windows.
    // Subtract,
    // /// Decimal key, `VK_DECIMAL` on Windows.
    // Decimal,
    // /// Divide key, `VK_DIVIDE` on Windows.
    // Divide,
    /// F1 key
    F1,
    /// F1 key
    F2,
    /// F1 key
    F3,
    /// F1 key
    F4,
    /// F1 key
    F5,
    /// F1 key
    F6,
    /// F1 key
    F7,
    /// F1 key
    F8,
    /// F1 key
    F9,
    /// F1 key
    F10,
    /// F1 key
    F11,
    /// F1 key
    F12,
    /// F1 key
    F13,
    /// F1 key
    F14,
    /// F1 key
    F15,
    /// F1 key
    F16,
    /// F1 key
    F17,
    /// F1 key
    F18,
    /// F1 key
    F19,
    /// F20 key
    F20,
    /// F20 key
    F21,
    /// F20 key
    F22,
    /// F20 key
    F23,
    /// F20 key
    F24,
    // /// NUM LOCK key
    // NumLock,
    // /// SCROLL LOCK key
    // ScrollLock,
    /// Browser Back key, `VK_BROWSER_BACK` on Windows.
    BrowserBack,
    /// Browser Forward key
    BrowserForward,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `;:` key
    Semicolon,
    /// For any country/region, the `+` key
    Plus,
    /// For any country/region, the `,` key
    Comma,
    /// For any country/region, the `-` key
    Minus,
    /// For any country/region, the . key
    Period,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `/?` key
    Slash,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `~ key
    Tilde,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `[{` key
    LeftBracket,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `\|` key
    Backslash,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `]}` key
    RightBracket,
    /// Used for miscellaneous characters, it can vary by keyboard.
    /// For the US standard keyboard, the `'"` key
    Quote,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM8,
    /// The `<>` keys on the US standard keyboard, or the `\|` key on the
    /// non-US 102-key keyboard
    OEM102,
}

/// TODO:
#[derive(Copy, Clone, Debug, Default)]
pub enum KeyPosition {
    /// TODO:
    #[default]
    Any,
    /// TODO:
    Left,
    /// TODO:
    Right,
}

impl PartialEq for KeyPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (KeyPosition::Right, KeyPosition::Left) | (KeyPosition::Left, KeyPosition::Right) => {
                false
            }
            _ => true,
        }
    }
}

impl Eq for KeyPosition {}

impl std::hash::Hash for KeyPosition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            KeyPosition::Any => 0,
            KeyPosition::Left => 1,
            KeyPosition::Right => 2,
        }
        .hash(state)
    }
}

impl KeyCode {
    fn basic_parse(input: &str) -> Option<Self> {
        Some(match input {
            "fn" => Self::Function,
            "cancel" => Self::Cancel,
            "backspace" => Self::Backspace,
            "tab" => Self::Tab,
            "enter" => Self::Enter,
            "shift" => Self::Shift(KeyPosition::Any),
            "ctrl" => Self::Control(KeyPosition::Any),
            "alt" => Self::Alt(KeyPosition::Any),
            "capslock" => Self::Capital,
            "escape" => Self::Escape,
            "space" => Self::Space,
            "pageup" => Self::PageUp,
            "pagedown" => Self::PageDown,
            "end" => Self::End,
            "home" => Self::Home,
            "left" => Self::Left,
            "up" => Self::Up,
            "right" => Self::Right,
            "down" => Self::Down,
            // VirtualKeyCode::PrintScreen => "UnImplemented",
            "insert" => Self::Insert,
            "delete" => Self::Delete,
            "win" | "cmd" | "super" => Self::Platform(KeyPosition::Any),
            "menu" => Self::App, // TODO: Chrome use this as Fn key
            "a" => Self::A,
            "b" => Self::B,
            "c" => Self::C,
            "d" => Self::D,
            "e" => Self::E,
            "f" => Self::F,
            "g" => Self::G,
            "h" => Self::H,
            "i" => Self::I,
            "j" => Self::J,
            "k" => Self::K,
            "l" => Self::L,
            "m" => Self::M,
            "n" => Self::N,
            "o" => Self::O,
            "p" => Self::P,
            "q" => Self::Q,
            "r" => Self::R,
            "s" => Self::S,
            "t" => Self::T,
            "u" => Self::U,
            "v" => Self::V,
            "w" => Self::W,
            "x" => Self::X,
            "y" => Self::Y,
            "z" => Self::Z,
            // VirtualKeyCode::Numpad0 => "UnImplemented", // TODO: Handle numpad keys
            // VirtualKeyCode::Numpad1 => "UnImplemented",
            // VirtualKeyCode::Numpad2 => "UnImplemented",
            // VirtualKeyCode::Numpad3 => "UnImplemented",
            // VirtualKeyCode::Numpad4 => "UnImplemented",
            // VirtualKeyCode::Numpad5 => "UnImplemented",
            // VirtualKeyCode::Numpad6 => "UnImplemented",
            // VirtualKeyCode::Numpad7 => "UnImplemented",
            // VirtualKeyCode::Numpad8 => "UnImplemented",
            // VirtualKeyCode::Numpad9 => "UnImplemented",
            // VirtualKeyCode::Multiply => "UnImplemented",
            // VirtualKeyCode::Add => "UnImplemented",
            // VirtualKeyCode::Separator => "UnImplemented",
            // VirtualKeyCode::Subtract => "UnImplemented",
            // VirtualKeyCode::Decimal => "UnImplemented",
            // VirtualKeyCode::Divide => "UnImplemented",
            "f1" => Self::F1,
            "f2" => Self::F2,
            "f3" => Self::F3,
            "f4" => Self::F4,
            "f5" => Self::F5,
            "f6" => Self::F6,
            "f7" => Self::F7,
            "f8" => Self::F8,
            "f9" => Self::F9,
            "f10" => Self::F10,
            "f11" => Self::F11,
            "f12" => Self::F12,
            "f13" => Self::F13,
            "f14" => Self::F14,
            "f15" => Self::F15,
            "f16" => Self::F16,
            "f17" => Self::F17,
            "f18" => Self::F18,
            "f19" => Self::F19,
            "f20" => Self::F20,
            "f21" => Self::F21,
            "f22" => Self::F22,
            "f23" => Self::F23,
            "f24" => Self::F24,
            "back" => Self::BrowserBack,
            "forward" => Self::BrowserForward,
            _ => return None,
        })
    }
    /// input is standard US English layout key
    pub fn parse(input: &str) -> anyhow::Result<Self> {
        if let Some(key) = Self::basic_parse(input) {
            return Ok(key);
        }
        match input {
            "0" => Ok(Self::Digital0),
            "1" => Ok(Self::Digital1),
            "2" => Ok(Self::Digital2),
            "3" => Ok(Self::Digital3),
            "4" => Ok(Self::Digital4),
            "5" => Ok(Self::Digital5),
            "6" => Ok(Self::Digital6),
            "7" => Ok(Self::Digital7),
            "8" => Ok(Self::Digital8),
            "9" => Ok(Self::Digital9),
            ";" => Ok(Self::Semicolon),
            "=" => Ok(Self::Plus),
            "," => Ok(Self::Comma),
            "-" => Ok(Self::Minus),
            "." => Ok(Self::Period),
            "/" => Ok(Self::Slash),
            "`" => Ok(Self::Tilde),
            "[" => Ok(Self::LeftBracket),
            "\\" => Ok(Self::Backslash),
            "]" => Ok(Self::RightBracket),
            "'" => Ok(Self::Quote),
            _ => Err(anyhow::anyhow!(
                "Error parsing keystroke to virtual keycode: {input}"
            )),
        }
    }

    // /// TODO:
    // fn parse_char(input: &str) -> anyhow::Result<(Self, bool, bool, bool)> {
    //     if let Some(key) = Self::basic_parse(input) {
    //         return Ok((key, false, false, false));
    //     }
    //     if input.chars().count() != 1 {
    //         return Err(anyhow::anyhow!(
    //             "Error parsing keystroke to virtual keycode (char based): {input}"
    //         ));
    //     }
    //     let ch = input.chars().next().unwrap();
    //     let result = unsafe { VkKeyScanW(ch as u16) };
    //     if result == -1 {
    //         return Err(anyhow::anyhow!(
    //             "Error parsing keystroke to virtual keycode (char based): {input}"
    //         ));
    //     }
    //     let high = (result >> 8) as u8;
    //     let low = result as u8;
    //     let shift = high & 1;
    //     let ctrl = (high >> 1) & 1;
    //     let alt = (high >> 2) & 1;
    //     let this = VIRTUAL_KEY(low as u16).try_into()?;
    //     Ok((this, shift != 0, ctrl != 0, alt != 0))
    // }

    // /// TODO:
    // pub fn unparse(&self) -> &str {
    //     match self {
    //         Self::Unknown(content) => &content,
    //         Self::Function => "fn",
    //         Self::Cancel => "cancel",
    //         Self::Backspace => "backspace",
    //         Self::Tab => "tab",
    //         Self::Clear => "UnImplemented",
    //         Self::Enter => "enter",
    //         // TODO: position
    //         Self::Shift(_) => "shift",
    //         Self::Control(_) => "ctrl",
    //         Self::Alt(_) => "alt",
    //         Self::Pause => "UnImplemented",
    //         Self::Capital => "capslock",
    //         // Self::Kana => "UnImplemented",
    //         // Self::Hangul => "UnImplemented",
    //         // Self::Junja => "UnImplemented",
    //         // Self::Final => "UnImplemented",
    //         // Self::Hanja => "UnImplemented",
    //         // Self::Kanji => "UnImplemented",
    //         Self::Escape => "escape",
    //         Self::Convert => "UnImplemented",
    //         Self::Nonconvert => "UnImplemented",
    //         Self::Accept => "UnImplemented",
    //         Self::ModeChange => "UnImplemented",
    //         Self::Space => "space",
    //         Self::PageUp => "pageup",
    //         Self::PageDown => "pagedown",
    //         Self::End => "end",
    //         Self::Home => "home",
    //         Self::Left => "left",
    //         Self::Up => "up",
    //         Self::Right => "right",
    //         Self::Down => "down",
    //         Self::Select => "UnImplemented",
    //         Self::Print => "UnImplemented",
    //         Self::Execute => "UnImplemented",
    //         Self::PrintScreen => "UnImplemented",
    //         Self::Insert => "insert",
    //         Self::Delete => "delete",
    //         Self::Help => "UnImplemented",
    //         Self::Digital0 => "0",
    //         Self::Digital1 => "1",
    //         Self::Digital2 => "2",
    //         Self::Digital3 => "3",
    //         Self::Digital4 => "4",
    //         Self::Digital5 => "5",
    //         Self::Digital6 => "6",
    //         Self::Digital7 => "7",
    //         Self::Digital8 => "8",
    //         Self::Digital9 => "9",
    //         Self::A => "a",
    //         Self::B => "b",
    //         Self::C => "c",
    //         Self::D => "d",
    //         Self::E => "e",
    //         Self::F => "f",
    //         Self::G => "g",
    //         Self::H => "h",
    //         Self::I => "i",
    //         Self::J => "j",
    //         Self::K => "k",
    //         Self::L => "l",
    //         Self::M => "m",
    //         Self::N => "n",
    //         Self::O => "o",
    //         Self::P => "p",
    //         Self::Q => "q",
    //         Self::R => "r",
    //         Self::S => "s",
    //         Self::T => "t",
    //         Self::U => "u",
    //         Self::V => "v",
    //         Self::W => "w",
    //         Self::X => "x",
    //         Self::Y => "y",
    //         Self::Z => "z",
    //         // TODO: handle position
    //         Self::Platform(_) => "win",
    //         Self::App => "menu", // TODO: Chrome use this as Fn key
    //         Self::Sleep => "UnImplemented",
    //         Self::Numpad0 => "UnImplemented", // TODO: handle numpad key
    //         Self::Numpad1 => "UnImplemented",
    //         Self::Numpad2 => "UnImplemented",
    //         Self::Numpad3 => "UnImplemented",
    //         Self::Numpad4 => "UnImplemented",
    //         Self::Numpad5 => "UnImplemented",
    //         Self::Numpad6 => "UnImplemented",
    //         Self::Numpad7 => "UnImplemented",
    //         Self::Numpad8 => "UnImplemented",
    //         Self::Numpad9 => "UnImplemented",
    //         Self::Multiply => "UnImplemented",
    //         Self::Add => "UnImplemented",
    //         Self::Separator => "UnImplemented",
    //         Self::Subtract => "UnImplemented",
    //         Self::Decimal => "UnImplemented",
    //         Self::Divide => "UnImplemented",
    //         Self::F1 => "f1",
    //         Self::F2 => "f2",
    //         Self::F3 => "f3",
    //         Self::F4 => "f4",
    //         Self::F5 => "f5",
    //         Self::F6 => "f6",
    //         Self::F7 => "f7",
    //         Self::F8 => "f8",
    //         Self::F9 => "f9",
    //         Self::F10 => "f10",
    //         Self::F11 => "f11",
    //         Self::F12 => "f12",
    //         Self::F13 => "f13",
    //         Self::F14 => "f14",
    //         Self::F15 => "f15",
    //         Self::F16 => "f16",
    //         Self::F17 => "f17",
    //         Self::F18 => "f18",
    //         Self::F19 => "f19",
    //         Self::F20 => "f20",
    //         Self::F21 => "f21",
    //         Self::F22 => "f22",
    //         Self::F23 => "f23",
    //         Self::F24 => "f24",
    //         Self::NumLock => "UnImplemented",
    //         Self::ScrollLock => "UnImplemented",
    //         Self::BrowserBack => "back",
    //         Self::BrowserForward => "forward",
    //         Self::BrowserRefresh => "UnImplemented",
    //         Self::BrowserStop => "UnImplemented",
    //         Self::BrowserSearch => "UnImplemented",
    //         Self::BrowserFavorites => "UnImplemented",
    //         Self::BrowserHome => "UnImplemented",
    //         Self::VolumeMute => "UnImplemented",
    //         Self::VolumeDown => "UnImplemented",
    //         Self::VolumeUp => "UnImplemented",
    //         Self::MediaNextTrack => "UnImplemented",
    //         Self::MediaPrevTrack => "UnImplemented",
    //         Self::MediaStop => "UnImplemented",
    //         Self::MediaPlayPause => "UnImplemented",
    //         Self::LaunchMail => "UnImplemented",
    //         Self::LaunchMediaSelect => "UnImplemented",
    //         Self::LaunchApp1 => "UnImplemented",
    //         Self::LaunchApp2 => "UnImplemented",
    //         Self::Semicolon => ";",
    //         Self::Plus => "=",
    //         Self::Comma => ",",
    //         Self::Minus => "-",
    //         Self::Period => ".",
    //         Self::Slash => "/",
    //         Self::Tilde => "`",
    //         Self::LeftBracket => "[",
    //         Self::Backslash => "\\",
    //         Self::RightBracket => "]",
    //         Self::Quote => "'",
    //         Self::OEM8 => "UnImplemented",
    //         Self::OEM102 => "UnImplemented",
    //         // Self::ProcessKey => "UnImplemented",
    //         // Self::Packet => "UnImplemented",
    //         // Self::Attn => "UnImplemented",
    //         // Self::CrSel => "UnImplemented",
    //         // Self::ExSel => "UnImplemented",
    //         // Self::EraseEOF => "UnImplemented",
    //         // Self::Play => "UnImplemented",
    //         // Self::Zoom => "UnImplemented",
    //         // Self::PA1 => "UnImplemented",
    //         // Self::OEMClear => "UnImplemented",
    //     }
    // }
    pub fn is_printable(&self) -> bool {
        !matches!(
            self,
            Self::F1
                | Self::F2
                | Self::F3
                | Self::F4
                | Self::F5
                | Self::F6
                | Self::F7
                | Self::F8
                | Self::F9
                | Self::F10
                | Self::F11
                | Self::F12
                | Self::F13
                | Self::F14
                | Self::F15
                | Self::F16
                | Self::F17
                | Self::F18
                | Self::F19
                | Self::F20
                | Self::F21
                | Self::F22
                | Self::F23
                | Self::F24
                | Self::Backspace
                | Self::Delete
                | Self::Left
                | Self::Up
                | Self::Right
                | Self::Down
                | Self::PageUp
                | Self::PageDown
                | Self::Insert
                | Self::Home
                | Self::End
                | Self::BrowserBack
                | Self::BrowserForward
                | Self::Escape
        )
    }
}

// static KEYBOARD_CODES: [KeyCode; 128] = [
//     KeyCode::A, // 0x00
//     KeyCode::S,
//     KeyCode::D,
//     KeyCode::F,
//     KeyCode::H,
//     KeyCode::G,
//     KeyCode::Z,
//     KeyCode::X,
//     KeyCode::C,
//     KeyCode::V,
//     KeyCode::Unknown, // Section key
//     KeyCode::B,
//     KeyCode::Q,
//     KeyCode::W,
//     KeyCode::E,
//     KeyCode::R,
//     KeyCode::Y,
//     KeyCode::T,
//     KeyCode::Digital1,
//     KeyCode::Digital2,
//     KeyCode::Digital3,
//     KeyCode::Digital4,
//     KeyCode::Digital6,
//     KeyCode::Digital5,
//     KeyCode::Plus, // =+
//     KeyCode::Digital9,
//     KeyCode::Digital7,
//     KeyCode::Minus, // -_
//     KeyCode::Digital8,
//     KeyCode::Digital0,
//     KeyCode::RightBracket, // ]}
//     KeyCode::O,
//     KeyCode::U,
//     KeyCode::LeftBracket, // [{
//     KeyCode::I,
//     KeyCode::P,
//     KeyCode::Enter,
//     KeyCode::L,
//     KeyCode::J,
//     KeyCode::Quote, // '"
//     KeyCode::K,
//     KeyCode::Semicolon, // ;:
//     KeyCode::Backslash, // \|
//     KeyCode::Comma,     // ,<
//     KeyCode::Slash,     // /?
//     KeyCode::N,
//     KeyCode::M,
//     KeyCode::Period, // .>
//     KeyCode::Tab,
//     KeyCode::Space,
//     KeyCode::Tilde, // `~
//     KeyCode::Backspace,
//     KeyCode::Unknown, // n/a
//     KeyCode::Escape,
//     KeyCode::App, // Right command
//     KeyCode::Platform(KeyPosition::Left),
//     KeyCode::Shift(KeyPosition::Left),
//     KeyCode::Capital,                     // Capslock
//     KeyCode::Alt(KeyPosition::Left),      // Left option
//     KeyCode::Control(KeyPosition::Left),  // Left control
//     KeyCode::Shift(KeyPosition::Right),   // Right shift
//     KeyCode::Alt(KeyPosition::Right),     // Right option
//     KeyCode::Control(KeyPosition::Right), // Right control
//     KeyCode::Function,                    // TODO: VK_UNKNOWN on Chrome
//     KeyCode::F17,
//     KeyCode::Decimal,  // Numpad .
//     KeyCode::Unknown,  // n/a
//     KeyCode::Multiply, // Numpad *
//     KeyCode::Unknown,  // n/a
//     KeyCode::Add,      // Numpad +
//     KeyCode::Unknown,  // n/a
//     KeyCode::Clear,    // Numpad clear
//     KeyCode::VolumeUp,
//     KeyCode::VolumeDown,
//     KeyCode::VolumeMute,
//     KeyCode::Divide,   // Numpad /
//     KeyCode::Enter,    // Numpad enter
//     KeyCode::Unknown,  // n/a
//     KeyCode::Subtract, // Numpad -
//     KeyCode::F18,
//     KeyCode::F19,
//     KeyCode::Plus, // Numpad =.
//     KeyCode::Numpad0,
//     KeyCode::Numpad1,
//     KeyCode::Numpad2,
//     KeyCode::Numpad3,
//     KeyCode::Numpad4,
//     KeyCode::Numpad5,
//     KeyCode::Numpad6,
//     KeyCode::Numpad7,
//     KeyCode::F20,
//     KeyCode::Numpad8,
//     KeyCode::Numpad9,
//     KeyCode::Unknown, // Yen, JIS keyboad only
//     KeyCode::Unknown, // Underscore, JIS keyboard only
//     KeyCode::Unknown, // Keypad comma, JIS keyboard only
//     KeyCode::F5,
//     KeyCode::F6,
//     KeyCode::F7,
//     KeyCode::F3,
//     KeyCode::F8,
//     KeyCode::F9,
//     KeyCode::Unknown, // Eisu, JIS keyboard only
//     KeyCode::F11,
//     KeyCode::Unknown, // Kana, JIS keyboard only
//     KeyCode::F13,
//     KeyCode::F16,
//     KeyCode::F14,
//     KeyCode::Unknown, // n/a
//     KeyCode::F10,
//     KeyCode::App, // Context menu key
//     KeyCode::F12,
//     KeyCode::Unknown, // n/a
//     KeyCode::F15,
//     KeyCode::Insert, // Help
//     KeyCode::Home,   // Home
//     KeyCode::PageUp,
//     KeyCode::Delete, // Forward delete
//     KeyCode::F4,
//     KeyCode::End,
//     KeyCode::F2,
//     KeyCode::PageDown,
//     KeyCode::F1,
//     KeyCode::Left,
//     KeyCode::Right,
//     KeyCode::Down,
//     KeyCode::Up,
//     KeyCode::Unknown, // n/a
// ];
