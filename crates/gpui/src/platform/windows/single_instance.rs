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

use super::APP_SHARED_MEMORY_MAX_SIZE;

static APP_IDENTIFIER: RwLock<String> = RwLock::new(String::new());
static APP_INSTANCE_EVENT_IDENTIFIER: RwLock<String> = RwLock::new(String::new());
static APP_SHARED_MEMORY_IDENTIFIER: RwLock<String> = RwLock::new(String::new());

pub(crate) fn register_app_identifier(app_identifier: &str, local: bool) {
    if app_identifier.len() as u32 > MAX_PATH {
        panic!(
            "The length of app identifier `{app_identifier}` is limited to {MAX_PATH} characters."
        );
    }
    *APP_IDENTIFIER.write() = app_identifier.to_string();
    let (sync_event_identifier, shared_memory_identifier) = if local {
        (
            format!("Local\\{app_identifier}-Instance-Event"),
            format!("Local\\{app_identifier}-Shared-Memory"),
        )
    } else {
        (
            format!("Global\\{app_identifier}-Instance-Event"),
            format!("Global\\{app_identifier}-Shared-Memory"),
        )
    };
    if sync_event_identifier.len() as u32 > MAX_PATH {
        panic!("The length of app identifier `{sync_event_identifier}` is limited to {MAX_PATH} characters.");
    }
    *APP_INSTANCE_EVENT_IDENTIFIER.write() = sync_event_identifier;
    if shared_memory_identifier.len() as u32 > MAX_PATH {
        panic!("The length of app identifier `{shared_memory_identifier}` is limited to {MAX_PATH} characters.");
    }
    *APP_SHARED_MEMORY_IDENTIFIER.write() = shared_memory_identifier;
}

pub(crate) fn retrieve_app_identifier() -> String {
    let lock = APP_IDENTIFIER.read();
    if lock.is_empty() {
        panic!("Make sure you have called `register_app_identifier` first.");
    }
    lock.to_string()
}

pub(crate) fn retrieve_app_instance_event_identifier() -> String {
    let lock = APP_INSTANCE_EVENT_IDENTIFIER.read();
    if lock.is_empty() {
        panic!("Make sure you have called `register_app_identifier` first.");
    }
    lock.to_string()
}

pub(crate) fn retrieve_app_shared_memory_identifier() -> String {
    let lock = APP_SHARED_MEMORY_IDENTIFIER.read();
    if lock.is_empty() {
        panic!("Make sure you have called `register_app_identifier` first.");
    }
    lock.to_string()
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
            &HSTRING::from(retrieve_app_instance_event_identifier()),
        )
        .expect("Unable to create instance sync event")
    };
    let last_err = unsafe { GetLastError() };
    let is_single_instance = last_err != ERROR_ALREADY_EXISTS;
    println!("-> Raw instance: {}", is_single_instance);
    f(is_single_instance)
}

pub(crate) fn send_message_to_other_instance() {
    let msg = format!("Hello from closed instance via PID {}", std::process::id());
    println!("=> sending: {}", msg);
    send_message_through_pipes(&msg);
    unsafe {
        let handle = OpenEventW(
            EVENT_MODIFY_STATE,
            false,
            &HSTRING::from(retrieve_app_instance_event_identifier()),
        )
        .unwrap();
        SetEvent(handle).log_err();
    }
}

fn send_message_through_pipes(message: &str) {
    if message.len() > APP_SHARED_MEMORY_MAX_SIZE {
        log::error!(
            "The length of the message to send should be less than {APP_SHARED_MEMORY_MAX_SIZE}"
        );
        return;
    }
    unsafe {
        let msg = message.as_bytes();
        let pipe = OpenFileMappingW(
            FILE_MAP_WRITE.0,
            false,
            &HSTRING::from(retrieve_app_shared_memory_identifier()),
        )
        .unwrap();
        let memory_addr = MapViewOfFile(pipe, FILE_MAP_WRITE, 0, 0, 0);
        std::ptr::copy_nonoverlapping(msg.as_ptr(), memory_addr.Value as _, msg.len());
        UnmapViewOfFile(memory_addr).log_err();
        CloseHandle(pipe).log_err();
    }
}
