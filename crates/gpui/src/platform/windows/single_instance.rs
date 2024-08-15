use parking_lot::RwLock;
use util::ResultExt;
use windows::Win32::{
    Foundation::{GetLastError, ERROR_ALREADY_EXISTS, GENERIC_READ, GENERIC_WRITE, MAX_PATH},
    Storage::FileSystem::{
        CreateFileW, OpenFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_WRITE_ATTRIBUTES,
        OPEN_EXISTING,
    },
    System::Threading::{CreateEventW, CreateMutexW, OpenEventW, SetEvent, EVENT_MODIFY_STATE},
    UI::WindowsAndMessaging::FindWindowW,
};
use windows_core::HSTRING;

static APP_IDENTIFIER: RwLock<String> = RwLock::new(String::new());
static APP_EVENT_IDENTIFIER: RwLock<String> = RwLock::new(String::new());

pub(crate) fn retrieve_app_identifier() -> String {
    let lock = APP_IDENTIFIER.read();
    if lock.is_empty() {
        panic!("Make sure you have called `register_app_identifier` first.");
    }
    lock.to_string()
}

pub(crate) fn register_app_identifier(app_identifier: &str, local: bool) {
    if app_identifier.len() as u32 > MAX_PATH {
        panic!(
            "The length of app identifier `{app_identifier}` is limited to {MAX_PATH} characters."
        );
    }
    *APP_IDENTIFIER.write() = app_identifier.to_string();
    let identifier = if local {
        format!("Local\\{app_identifier}")
    } else {
        format!("Global\\{app_identifier}")
    };
    if identifier.len() as u32 > MAX_PATH {
        panic!("The length of app identifier `{identifier}` is limited to {MAX_PATH} characters.");
    }
    *APP_EVENT_IDENTIFIER.write() = identifier;
}

pub(crate) fn retrieve_app_event_identifier() -> String {
    let lock = APP_EVENT_IDENTIFIER.read();
    if lock.is_empty() {
        panic!("Make sure you have called `check_single_instance` first.");
    }
    lock.to_string()
}

pub(crate) fn retrieve_named_pipe_identifier() -> String {
    format!("\\\\.\\pipe\\{}", APP_IDENTIFIER.read())
}

pub(crate) fn check_single_instance<F>(f: F) -> bool
where
    F: FnOnce(bool) -> bool,
{
    let identifier = retrieve_app_event_identifier();
    unsafe {
        // CreateMutexW(None, true, &HSTRING::from(identifier.as_str())).unwrap_or_else(|_| {
        //     panic!(
        //         "Unable to create instance sync event!\n{:?}",
        //         std::io::Error::last_os_error()
        //     )
        // })
        CreateEventW(None, false, false, &HSTRING::from(identifier.as_str())).unwrap_or_else(|_| {
            panic!(
                "Unable to create instance sync event!\n{:?}",
                std::io::Error::last_os_error()
            )
        })
    };
    let last_err = unsafe { GetLastError() };
    let is_single_instance = last_err != ERROR_ALREADY_EXISTS;
    println!("-> Raw instance: {}", is_single_instance);
    f(is_single_instance)
}

pub(crate) fn send_message_to_other_instance() {
    // let handle = unsafe { FindWindowW(&HSTRING::from(retrieve_app_identifier()), None).unwrap() };
    // assert!(!handle.is_invalid());
    let msg = "Hello from closed instance".to_owned();
    println!("=> sending: {}", msg);
    unsafe {
        send_message_through_pipes();
        let handle = OpenEventW(
            EVENT_MODIFY_STATE,
            false,
            &HSTRING::from(APP_IDENTIFIER.read().as_str()),
        )
        .unwrap();
        SetEvent(handle).log_err();
    }
}

fn send_message_through_pipes() {
    unsafe {
        let pipe = CreateFileW(
            &HSTRING::from(retrieve_named_pipe_identifier()),
            GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
        .unwrap();
    }
}
