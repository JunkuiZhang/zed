use parking_lot::RwLock;
use util::ResultExt;
use windows::Win32::{
    Foundation::{GetLastError, ERROR_ALREADY_EXISTS, MAX_PATH},
    System::Threading::{CreateEventW, OpenEventW, SetEvent, EVENT_MODIFY_STATE},
};
use windows_core::HSTRING;

pub(crate) static APP_IDENTIFIER: RwLock<String> = RwLock::new(String::new());

pub(crate) fn check_single_instance<F>(app_identifier: &str, local: bool, f: F) -> bool
where
    F: FnOnce(bool) -> bool,
{
    let identifier = if local {
        format!("Local\\{app_identifier}")
    } else {
        format!("Global\\{app_identifier}")
    };
    if identifier.len() as u32 > MAX_PATH {
        panic!("The length of app identifier is limited to {MAX_PATH} characters.");
    }
    *APP_IDENTIFIER.write() = identifier.clone();
    unsafe {
        CreateEventW(None, false, false, &HSTRING::from(identifier.as_str())).expect(
            format!(
                "Unable to create instance sync event!\n{:?}",
                std::io::Error::last_os_error()
            )
            .as_str(),
        )
    };
    let last_err = unsafe { GetLastError() };
    let is_single_instance = last_err != ERROR_ALREADY_EXISTS;
    println!("-> Raw instance: {}", is_single_instance);
    f(is_single_instance)
}

pub(crate) fn send_message_to_other_instance() {
    unsafe {
        let handle = OpenEventW(
            EVENT_MODIFY_STATE,
            false,
            &HSTRING::from(APP_IDENTIFIER.read().as_str()),
        )
        .unwrap();
        SetEvent(handle).log_err();
    }
}
