use parking_lot::RwLock;
use util::ResultExt;
use windows::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, MAX_PATH},
    System::{
        Memory::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_WRITE},
        Threading::{CreateEventW, OpenEventW, SetEvent, EVENT_MODIFY_STATE},
    },
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

pub(crate) fn retrieve_shared_memory_identifier() -> String {
    format!("Shared-Memory-{}", APP_IDENTIFIER.read())
}

pub(crate) fn check_single_instance<F>(f: F) -> bool
where
    F: FnOnce(bool) -> bool,
{
    unsafe {
        CreateEventW(
            None,
            false,
            false,
            &HSTRING::from(retrieve_app_event_identifier()),
        )
        .expect("Unable to create instance sync event")
    };
    let last_err = unsafe { GetLastError() };
    let is_single_instance = last_err != ERROR_ALREADY_EXISTS;
    println!("-> Raw instance: {}", is_single_instance);
    f(is_single_instance)
}

pub(crate) fn send_message_to_other_instance() {
    let msg = "Hello from closed instance";
    println!("=> sending: {}", msg);
    send_message_through_pipes(msg);
    unsafe {
        let handle = OpenEventW(
            EVENT_MODIFY_STATE,
            false,
            &HSTRING::from(retrieve_app_event_identifier()),
        )
        .unwrap();
        SetEvent(handle).log_err();
    }
}

fn send_message_through_pipes(message: &str) {
    unsafe {
        let msg = message.as_bytes();
        let pipe = OpenFileMappingW(
            FILE_MAP_WRITE.0,
            false,
            &HSTRING::from(retrieve_shared_memory_identifier()),
        )
        .unwrap();
        let memory_addr = MapViewOfFile(pipe, FILE_MAP_WRITE, 0, 0, 0);
        std::ptr::copy_nonoverlapping(msg.as_ptr(), memory_addr.Value as _, msg.len());
        let _ = UnmapViewOfFile(memory_addr).log_err();
        let _ = CloseHandle(pipe).log_err();
    }
}
