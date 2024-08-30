use std::{collections::HashMap, sync::LazyLock};

use crate::VirtualKeyCode;

pub(crate) static GERMAN_LAYOUT_ANSI: LazyLock<HashMap<u16, VirtualKeyCode>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert(0x32, VirtualKeyCode::OEM5);
        map.insert(0x1B, VirtualKeyCode::OEM4);
        map.insert(0x18, VirtualKeyCode::OEM6);
        map.insert(0x21, VirtualKeyCode::OEM1);
        map.insert(0x2A, VirtualKeyCode::OEM2);
        map.insert(0x29, VirtualKeyCode::OEM3);
        map.insert(0x27, VirtualKeyCode::OEM7);
        map
    });

pub(crate) static GERMAN_LAYOUT_ISO: LazyLock<HashMap<u16, VirtualKeyCode>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0x0A, VirtualKeyCode::OEM5);
    map.insert(0x1B, VirtualKeyCode::OEM4);
    map.insert(0x18, VirtualKeyCode::OEM6);
    map.insert(0x21, VirtualKeyCode::OEM1);
    map.insert(0x2A, VirtualKeyCode::OEM2);
    map.insert(0x29, VirtualKeyCode::OEM3);
    map.insert(0x27, VirtualKeyCode::OEM7);
    map.insert(0x32, VirtualKeyCode::OEM102);
    map
});
