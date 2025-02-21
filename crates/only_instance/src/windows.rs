use release_channel::ReleaseChannel;
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateEventW,
    },
};

pub fn ensure_only_instance() -> bool {
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
    last_err != ERROR_ALREADY_EXISTS
}

fn retrieve_app_instance_event_identifier() -> &'static str {
    match *release_channel::RELEASE_CHANNEL {
        ReleaseChannel::Dev => "Local\\Zed-Editor-Dev-Instance-Event",
        ReleaseChannel::Nightly => "Local\\Zed-Editor-Nightly-Instance-Event",
        ReleaseChannel::Preview => "Local\\Zed-Editor-Preview-Instance-Event",
        ReleaseChannel::Stable => "Local\\Zed-Editor-Stable-Instance-Event",
    }
}
