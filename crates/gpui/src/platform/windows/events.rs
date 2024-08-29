use std::{cell::RefMut, rc::Rc};

use ::util::ResultExt;
use anyhow::Context;
use keycodes::VirtualKeyCode;
use windows::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::SystemServices::*,
    UI::{
        HiDpi::*,
        Input::{Ime::*, KeyboardAndMouse::*},
        WindowsAndMessaging::*,
    },
};

use crate::*;

pub(crate) const CURSOR_STYLE_CHANGED: u32 = WM_USER + 1;
pub(crate) const CLOSE_ONE_WINDOW: u32 = WM_USER + 2;

const SIZE_MOVE_LOOP_TIMER_ID: usize = 1;
const AUTO_HIDE_TASKBAR_THICKNESS_PX: i32 = 1;

pub(crate) fn handle_msg(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> LRESULT {
    let handled = match msg {
        WM_ACTIVATE => handle_activate_msg(handle, wparam, state_ptr),
        WM_CREATE => handle_create_msg(handle, state_ptr),
        WM_MOVE => handle_move_msg(handle, lparam, state_ptr),
        WM_SIZE => handle_size_msg(lparam, state_ptr),
        WM_ENTERSIZEMOVE | WM_ENTERMENULOOP => handle_size_move_loop(handle),
        WM_EXITSIZEMOVE | WM_EXITMENULOOP => handle_size_move_loop_exit(handle),
        WM_TIMER => handle_timer_msg(handle, wparam, state_ptr),
        WM_NCCALCSIZE => handle_calc_client_size(handle, wparam, lparam, state_ptr),
        WM_DPICHANGED => handle_dpi_changed_msg(handle, wparam, lparam, state_ptr),
        WM_DISPLAYCHANGE => handle_display_change_msg(handle, state_ptr),
        WM_NCHITTEST => handle_hit_test_msg(handle, msg, wparam, lparam, state_ptr),
        WM_PAINT => handle_paint_msg(handle, state_ptr),
        WM_CLOSE => handle_close_msg(state_ptr),
        WM_DESTROY => handle_destroy_msg(handle, state_ptr),
        WM_MOUSEMOVE => handle_mouse_move_msg(lparam, wparam, state_ptr),
        WM_NCMOUSEMOVE => handle_nc_mouse_move_msg(handle, lparam, state_ptr),
        WM_NCLBUTTONDOWN => {
            handle_nc_mouse_down_msg(handle, MouseButton::Left, wparam, lparam, state_ptr)
        }
        WM_NCRBUTTONDOWN => {
            handle_nc_mouse_down_msg(handle, MouseButton::Right, wparam, lparam, state_ptr)
        }
        WM_NCMBUTTONDOWN => {
            handle_nc_mouse_down_msg(handle, MouseButton::Middle, wparam, lparam, state_ptr)
        }
        WM_NCLBUTTONUP => {
            handle_nc_mouse_up_msg(handle, MouseButton::Left, wparam, lparam, state_ptr)
        }
        WM_NCRBUTTONUP => {
            handle_nc_mouse_up_msg(handle, MouseButton::Right, wparam, lparam, state_ptr)
        }
        WM_NCMBUTTONUP => {
            handle_nc_mouse_up_msg(handle, MouseButton::Middle, wparam, lparam, state_ptr)
        }
        WM_LBUTTONDOWN => handle_mouse_down_msg(handle, MouseButton::Left, lparam, state_ptr),
        WM_RBUTTONDOWN => handle_mouse_down_msg(handle, MouseButton::Right, lparam, state_ptr),
        WM_MBUTTONDOWN => handle_mouse_down_msg(handle, MouseButton::Middle, lparam, state_ptr),
        WM_XBUTTONDOWN => {
            handle_xbutton_msg(handle, wparam, lparam, handle_mouse_down_msg, state_ptr)
        }
        WM_LBUTTONUP => handle_mouse_up_msg(handle, MouseButton::Left, lparam, state_ptr),
        WM_RBUTTONUP => handle_mouse_up_msg(handle, MouseButton::Right, lparam, state_ptr),
        WM_MBUTTONUP => handle_mouse_up_msg(handle, MouseButton::Middle, lparam, state_ptr),
        WM_XBUTTONUP => handle_xbutton_msg(handle, wparam, lparam, handle_mouse_up_msg, state_ptr),
        WM_MOUSEWHEEL => handle_mouse_wheel_msg(handle, wparam, lparam, state_ptr),
        WM_MOUSEHWHEEL => handle_mouse_horizontal_wheel_msg(handle, wparam, lparam, state_ptr),
        WM_SYSKEYDOWN => handle_syskeydown_msg(wparam, lparam, state_ptr),
        WM_SYSKEYUP => handle_syskeyup_msg(wparam, state_ptr),
        WM_SYSCOMMAND => handle_system_command(wparam, state_ptr),
        WM_KEYDOWN => handle_keydown_msg(wparam, lparam, state_ptr),
        WM_KEYUP => handle_keyup_msg(wparam, state_ptr),
        WM_CHAR => handle_char_msg(wparam, state_ptr),
        WM_IME_STARTCOMPOSITION => handle_ime_position(handle, state_ptr),
        WM_IME_COMPOSITION => handle_ime_composition(handle, lparam, state_ptr),
        WM_SETCURSOR => handle_set_cursor(lparam, state_ptr),
        WM_SETTINGCHANGE => handle_system_settings_changed(handle, state_ptr),
        WM_DWMCOLORIZATIONCOLORCHANGED => handle_system_theme_changed(state_ptr),
        CURSOR_STYLE_CHANGED => handle_cursor_changed(lparam, state_ptr),
        _ => None,
    };
    if let Some(n) = handled {
        LRESULT(n)
    } else {
        unsafe { DefWindowProcW(handle, msg, wparam, lparam) }
    }
}

fn handle_move_msg(
    handle: HWND,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let mut lock = state_ptr.state.borrow_mut();
    let origin = logical_point(
        lparam.signed_loword() as f32,
        lparam.signed_hiword() as f32,
        lock.scale_factor,
    );
    lock.origin = origin;
    let size = lock.logical_size;
    let center_x = origin.x.0 + size.width.0 / 2.;
    let center_y = origin.y.0 + size.height.0 / 2.;
    let monitor_bounds = lock.display.bounds();
    if center_x < monitor_bounds.left().0
        || center_x > monitor_bounds.right().0
        || center_y < monitor_bounds.top().0
        || center_y > monitor_bounds.bottom().0
    {
        // center of the window may have moved to another monitor
        let monitor = unsafe { MonitorFromWindow(handle, MONITOR_DEFAULTTONULL) };
        // minimize the window can trigger this event too, in this case,
        // monitor is invalid, we do nothing.
        if !monitor.is_invalid() && lock.display.handle != monitor {
            // we will get the same monitor if we only have one
            lock.display = WindowsDisplay::new_with_handle(monitor);
        }
    }
    if let Some(mut callback) = lock.callbacks.moved.take() {
        drop(lock);
        callback();
        state_ptr.state.borrow_mut().callbacks.moved = Some(callback);
    }
    Some(0)
}

fn handle_size_msg(lparam: LPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    let width = lparam.loword().max(1) as i32;
    let height = lparam.hiword().max(1) as i32;
    let mut lock = state_ptr.state.borrow_mut();
    let new_size = size(DevicePixels(width), DevicePixels(height));
    let scale_factor = lock.scale_factor;
    lock.renderer.update_drawable_size(new_size);
    let new_size = new_size.to_pixels(scale_factor);
    lock.logical_size = new_size;
    if let Some(mut callback) = lock.callbacks.resize.take() {
        drop(lock);
        callback(new_size, scale_factor);
        state_ptr.state.borrow_mut().callbacks.resize = Some(callback);
    }
    Some(0)
}

fn handle_size_move_loop(handle: HWND) -> Option<isize> {
    unsafe {
        let ret = SetTimer(handle, SIZE_MOVE_LOOP_TIMER_ID, USER_TIMER_MINIMUM, None);
        if ret == 0 {
            log::error!(
                "unable to create timer: {}",
                std::io::Error::last_os_error()
            );
        }
    }
    None
}

fn handle_size_move_loop_exit(handle: HWND) -> Option<isize> {
    unsafe {
        KillTimer(handle, SIZE_MOVE_LOOP_TIMER_ID).log_err();
    }
    None
}

fn handle_timer_msg(
    handle: HWND,
    wparam: WPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    if wparam.0 == SIZE_MOVE_LOOP_TIMER_ID {
        handle_paint_msg(handle, state_ptr)
    } else {
        None
    }
}

fn handle_paint_msg(handle: HWND, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    let mut lock = state_ptr.state.borrow_mut();
    if let Some(mut request_frame) = lock.callbacks.request_frame.take() {
        drop(lock);
        request_frame();
        state_ptr.state.borrow_mut().callbacks.request_frame = Some(request_frame);
    }
    unsafe { ValidateRect(handle, None).ok().log_err() };
    Some(0)
}

fn handle_close_msg(state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    let mut lock = state_ptr.state.borrow_mut();
    if let Some(mut callback) = lock.callbacks.should_close.take() {
        drop(lock);
        let should_close = callback();
        state_ptr.state.borrow_mut().callbacks.should_close = Some(callback);
        if should_close {
            None
        } else {
            Some(0)
        }
    } else {
        None
    }
}

fn handle_destroy_msg(handle: HWND, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    let callback = {
        let mut lock = state_ptr.state.borrow_mut();
        lock.callbacks.close.take()
    };
    if let Some(callback) = callback {
        callback();
    }
    unsafe {
        PostMessageW(
            None,
            CLOSE_ONE_WINDOW,
            WPARAM(state_ptr.validation_number),
            LPARAM(handle.0 as isize),
        )
        .log_err();
    }
    Some(0)
}

fn handle_mouse_move_msg(
    lparam: LPARAM,
    wparam: WPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let scale_factor = state_ptr.state.borrow().scale_factor;
    let pressed_button = match MODIFIERKEYS_FLAGS(wparam.loword() as u32) {
        flags if flags.contains(MK_LBUTTON) => Some(MouseButton::Left),
        flags if flags.contains(MK_RBUTTON) => Some(MouseButton::Right),
        flags if flags.contains(MK_MBUTTON) => Some(MouseButton::Middle),
        flags if flags.contains(MK_XBUTTON1) => {
            Some(MouseButton::Navigate(NavigationDirection::Back))
        }
        flags if flags.contains(MK_XBUTTON2) => {
            Some(MouseButton::Navigate(NavigationDirection::Forward))
        }
        _ => None,
    };
    let x = lparam.signed_loword() as f32;
    let y = lparam.signed_hiword() as f32;
    let event = PlatformInput::MouseMove(MouseMoveEvent {
        position: logical_point(x, y, scale_factor),
        pressed_button,
        modifiers: current_modifiers(),
    });
    with_keyboard_input_handler(&state_ptr, event, |_, _| {});
    Some(0)
}

fn handle_syskeydown_msg(
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    // we need to call `DefWindowProcW`, or we will lose the system-wide `Alt+F4`, `Alt+{other keys}`
    // shortcuts.
    let event = PlatformInput::KeyDown(KeyDownEvent {
        keystroke: parse_syskeydown_msg_keystroke(wparam)?,
        is_held: lparam.0 & (0x1 << 30) > 0,
    });
    if with_keyboard_input_handler(&state_ptr, event, |lock, handled| {
        lock.system_key_handled = handled;
    })
    .is_some()
    {
        Some(0)
    } else {
        None
    }
}

fn handle_syskeyup_msg(wparam: WPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    // we need to call `DefWindowProcW`, or we will lose the system-wide `Alt+F4`, `Alt+{other keys}`
    // shortcuts.
    let event = PlatformInput::KeyUp(KeyUpEvent {
        keystroke: parse_syskeydown_msg_keystroke(wparam)?,
    });
    if with_keyboard_input_handler(&state_ptr, event, |_, _| {}).is_some() {
        Some(0)
    } else {
        None
    }
}

fn handle_keydown_msg(
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    println!("WM_KEYDOWN");
    let event = parse_keydown_msg_to_platform_input(wparam, lparam);
    println!("char: {:?}, keycode: {}", event, wparam.0);
    with_keyboard_input_handler(&state_ptr, event, |lock, handled| {
        lock.last_keydown_handled = handled;
    });
    Some(0)
}

fn handle_keyup_msg(wparam: WPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    let event = parse_keydup_msg_to_platform_input(wparam);
    with_keyboard_input_handler(&state_ptr, event, |_, _| {});
    Some(0)
}

fn handle_char_msg(wparam: WPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    println!("WM_CHAR");
    if state_ptr.state.borrow().last_keydown_handled {
        return Some(0);
    }
    let Some(ime_char) = parse_char_msg(wparam) else {
        return Some(1);
    };
    with_input_handler(&state_ptr, |input_handler| {
        input_handler.replace_text_in_range(None, &ime_char);
    });

    Some(0)
}

fn handle_mouse_down_msg(
    handle: HWND,
    button: MouseButton,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    unsafe { SetCapture(handle) };
    let x = lparam.signed_loword() as f32;
    let y = lparam.signed_hiword() as f32;
    let physical_point = point(DevicePixels(x as i32), DevicePixels(y as i32));
    let mut lock = state_ptr.state.borrow_mut();
    let click_count = lock.click_state.update(button, physical_point);
    let scale_factor = lock.scale_factor;
    drop(lock);

    let event = PlatformInput::MouseDown(MouseDownEvent {
        button,
        position: logical_point(x, y, scale_factor),
        modifiers: current_modifiers(),
        click_count,
        first_mouse: false,
    });
    with_platform_input_handler(&state_ptr, event);
    Some(0)
}

fn handle_mouse_up_msg(
    _handle: HWND,
    button: MouseButton,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    unsafe { ReleaseCapture().log_err() };
    let x = lparam.signed_loword() as f32;
    let y = lparam.signed_hiword() as f32;
    let lock = state_ptr.state.borrow();
    let click_count = lock.click_state.current_count;
    let scale_factor = lock.scale_factor;
    drop(lock);

    let event = PlatformInput::MouseUp(MouseUpEvent {
        button,
        position: logical_point(x, y, scale_factor),
        modifiers: current_modifiers(),
        click_count,
    });
    with_platform_input_handler(&state_ptr, event);
    Some(0)
}

fn handle_xbutton_msg(
    handle: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    handler: impl Fn(HWND, MouseButton, LPARAM, Rc<WindowsWindowStatePtr>) -> Option<isize>,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let nav_dir = match wparam.hiword() {
        XBUTTON1 => NavigationDirection::Back,
        XBUTTON2 => NavigationDirection::Forward,
        _ => return Some(1),
    };
    handler(handle, MouseButton::Navigate(nav_dir), lparam, state_ptr)
}

fn handle_mouse_wheel_msg(
    handle: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let modifiers = current_modifiers();
    let lock = state_ptr.state.borrow();
    let scale_factor = lock.scale_factor;
    let wheel_scroll_amount = match modifiers.shift {
        true => lock.system_settings.mouse_wheel_settings.wheel_scroll_chars,
        false => lock.system_settings.mouse_wheel_settings.wheel_scroll_lines,
    };
    drop(lock);

    let wheel_distance =
        (wparam.signed_hiword() as f32 / WHEEL_DELTA as f32) * wheel_scroll_amount as f32;
    let mut cursor_point = POINT {
        x: lparam.signed_loword().into(),
        y: lparam.signed_hiword().into(),
    };
    unsafe { ScreenToClient(handle, &mut cursor_point).ok().log_err() };
    let event = PlatformInput::ScrollWheel(ScrollWheelEvent {
        position: logical_point(cursor_point.x as f32, cursor_point.y as f32, scale_factor),
        delta: ScrollDelta::Lines(match modifiers.shift {
            true => Point {
                x: wheel_distance,
                y: 0.0,
            },
            false => Point {
                y: wheel_distance,
                x: 0.0,
            },
        }),
        modifiers: current_modifiers(),
        touch_phase: TouchPhase::Moved,
    });
    with_platform_input_handler(&state_ptr, event);
    Some(0)
}

fn handle_mouse_horizontal_wheel_msg(
    handle: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let lock = state_ptr.state.borrow();
    let scale_factor = lock.scale_factor;
    let wheel_scroll_chars = lock.system_settings.mouse_wheel_settings.wheel_scroll_chars;
    drop(lock);
    let wheel_distance =
        (-wparam.signed_hiword() as f32 / WHEEL_DELTA as f32) * wheel_scroll_chars as f32;
    let mut cursor_point = POINT {
        x: lparam.signed_loword().into(),
        y: lparam.signed_hiword().into(),
    };
    unsafe { ScreenToClient(handle, &mut cursor_point).ok().log_err() };
    let event = PlatformInput::ScrollWheel(ScrollWheelEvent {
        position: logical_point(cursor_point.x as f32, cursor_point.y as f32, scale_factor),
        delta: ScrollDelta::Lines(Point {
            x: wheel_distance,
            y: 0.0,
        }),
        modifiers: current_modifiers(),
        touch_phase: TouchPhase::Moved,
    });
    with_platform_input_handler(&state_ptr, event);
    Some(0)
}

fn retrieve_caret_position(state_ptr: &Rc<WindowsWindowStatePtr>) -> Option<POINT> {
    with_input_handler_and_scale_factor(state_ptr, |input_handler, scale_factor| {
        let caret_range = input_handler.selected_text_range(false)?;
        let caret_position = input_handler.bounds_for_range(caret_range.range)?;
        Some(POINT {
            // logical to physical
            x: (caret_position.origin.x.0 * scale_factor) as i32,
            y: (caret_position.origin.y.0 * scale_factor) as i32
                + ((caret_position.size.height.0 * scale_factor) as i32 / 2),
        })
    })
}

fn handle_ime_position(handle: HWND, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    unsafe {
        let ctx = ImmGetContext(handle);

        let Some(caret_position) = retrieve_caret_position(&state_ptr) else {
            return Some(0);
        };
        {
            let config = COMPOSITIONFORM {
                dwStyle: CFS_POINT,
                ptCurrentPos: caret_position,
                ..Default::default()
            };
            ImmSetCompositionWindow(ctx, &config as _).ok().log_err();
        }
        {
            let config = CANDIDATEFORM {
                dwStyle: CFS_CANDIDATEPOS,
                ptCurrentPos: caret_position,
                ..Default::default()
            };
            ImmSetCandidateWindow(ctx, &config as _).ok().log_err();
        }
        ImmReleaseContext(handle, ctx).ok().log_err();
        Some(0)
    }
}

fn handle_ime_composition(
    handle: HWND,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let ctx = unsafe { ImmGetContext(handle) };
    let result = handle_ime_composition_inner(ctx, lparam, state_ptr);
    unsafe { ImmReleaseContext(handle, ctx).ok().log_err() };
    result
}

fn handle_ime_composition_inner(
    ctx: HIMC,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let mut ime_input = None;
    if lparam.0 as u32 & GCS_COMPSTR.0 > 0 {
        let (comp_string, string_len) = parse_ime_compostion_string(ctx)?;
        with_input_handler(&state_ptr, |input_handler| {
            input_handler.replace_and_mark_text_in_range(
                None,
                &comp_string,
                Some(string_len..string_len),
            );
        })?;
        ime_input = Some(comp_string);
    }
    if lparam.0 as u32 & GCS_CURSORPOS.0 > 0 {
        let comp_string = &ime_input?;
        let caret_pos = retrieve_composition_cursor_position(ctx);
        with_input_handler(&state_ptr, |input_handler| {
            input_handler.replace_and_mark_text_in_range(
                None,
                comp_string,
                Some(caret_pos..caret_pos),
            );
        })?;
    }
    if lparam.0 as u32 & GCS_RESULTSTR.0 > 0 {
        let comp_result = parse_ime_compostion_result(ctx)?;
        with_input_handler(&state_ptr, |input_handler| {
            input_handler.replace_text_in_range(None, &comp_result);
        })?;
        return Some(0);
    }
    // currently, we don't care other stuff
    None
}

/// SEE: https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-nccalcsize
fn handle_calc_client_size(
    handle: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    if !state_ptr.hide_title_bar || state_ptr.state.borrow().is_fullscreen() || wparam.0 == 0 {
        return None;
    }

    let is_maximized = state_ptr.state.borrow().is_maximized();
    let insets = get_client_area_insets(handle, is_maximized, state_ptr.windows_version);
    // wparam is TRUE so lparam points to an NCCALCSIZE_PARAMS structure
    let params = lparam.0 as *mut NCCALCSIZE_PARAMS;
    let requested_client_rect = unsafe { &mut ((*params).rgrc) };

    requested_client_rect[0].left += insets.left;
    requested_client_rect[0].top += insets.top;
    requested_client_rect[0].right -= insets.right;
    requested_client_rect[0].bottom -= insets.bottom;

    // Fix auto hide taskbar not showing. This solution is based on the approach
    // used by Chrome. However, it may result in one row of pixels being obscured
    // in our client area. But as Chrome says, "there seems to be no better solution."
    if is_maximized {
        if let Some(ref taskbar_position) = state_ptr
            .state
            .borrow()
            .system_settings
            .auto_hide_taskbar_position
        {
            // Fot the auto-hide taskbar, adjust in by 1 pixel on taskbar edge,
            // so the window isn't treated as a "fullscreen app", which would cause
            // the taskbar to disappear.
            match taskbar_position {
                AutoHideTaskbarPosition::Left => {
                    requested_client_rect[0].left += AUTO_HIDE_TASKBAR_THICKNESS_PX
                }
                AutoHideTaskbarPosition::Top => {
                    requested_client_rect[0].top += AUTO_HIDE_TASKBAR_THICKNESS_PX
                }
                AutoHideTaskbarPosition::Right => {
                    requested_client_rect[0].right -= AUTO_HIDE_TASKBAR_THICKNESS_PX
                }
                AutoHideTaskbarPosition::Bottom => {
                    requested_client_rect[0].bottom -= AUTO_HIDE_TASKBAR_THICKNESS_PX
                }
            }
        }
    }

    Some(0)
}

fn handle_activate_msg(
    handle: HWND,
    wparam: WPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let activated = wparam.loword() > 0;
    if state_ptr.hide_title_bar {
        if let Some(titlebar_rect) = state_ptr.state.borrow().get_titlebar_rect().log_err() {
            unsafe {
                InvalidateRect(handle, Some(&titlebar_rect), FALSE)
                    .ok()
                    .log_err()
            };
        }
    }
    let this = state_ptr.clone();
    state_ptr
        .executor
        .spawn(async move {
            let mut lock = this.state.borrow_mut();
            if let Some(mut cb) = lock.callbacks.active_status_change.take() {
                drop(lock);
                cb(activated);
                this.state.borrow_mut().callbacks.active_status_change = Some(cb);
            }
        })
        .detach();

    None
}

fn handle_create_msg(handle: HWND, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    if state_ptr.hide_title_bar {
        notify_frame_changed(handle);
        Some(0)
    } else {
        None
    }
}

fn handle_dpi_changed_msg(
    handle: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let new_dpi = wparam.loword() as f32;
    let mut lock = state_ptr.state.borrow_mut();
    lock.scale_factor = new_dpi / USER_DEFAULT_SCREEN_DPI as f32;
    lock.border_offset.update(handle).log_err();
    drop(lock);

    let rect = unsafe { &*(lparam.0 as *const RECT) };
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    // this will emit `WM_SIZE` and `WM_MOVE` right here
    // even before this function returns
    // the new size is handled in `WM_SIZE`
    unsafe {
        SetWindowPos(
            handle,
            None,
            rect.left,
            rect.top,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .context("unable to set window position after dpi has changed")
        .log_err();
    }

    Some(0)
}

/// The following conditions will trigger this event:
/// 1. The monitor on which the window is located goes offline or changes resolution.
/// 2. Another monitor goes offline, is plugged in, or changes resolution.
///
/// In either case, the window will only receive information from the monitor on which
/// it is located.
///
/// For example, in the case of condition 2, where the monitor on which the window is
/// located has actually changed nothing, it will still receive this event.
fn handle_display_change_msg(handle: HWND, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    // NOTE:
    // Even the `lParam` holds the resolution of the screen, we just ignore it.
    // Because WM_DPICHANGED, WM_MOVE, WM_SIZE will come first, window reposition and resize
    // are handled there.
    // So we only care about if monitor is disconnected.
    let previous_monitor = state_ptr.as_ref().state.borrow().display;
    if WindowsDisplay::is_connected(previous_monitor.handle) {
        // we are fine, other display changed
        return None;
    }
    // display disconnected
    // in this case, the OS will move our window to another monitor, and minimize it.
    // we deminimize the window and query the monitor after moving
    unsafe {
        let _ = ShowWindow(handle, SW_SHOWNORMAL);
    };
    let new_monitor = unsafe { MonitorFromWindow(handle, MONITOR_DEFAULTTONULL) };
    // all monitors disconnected
    if new_monitor.is_invalid() {
        log::error!("No monitor detected!");
        return None;
    }
    let new_display = WindowsDisplay::new_with_handle(new_monitor);
    state_ptr.as_ref().state.borrow_mut().display = new_display;
    Some(0)
}

fn handle_hit_test_msg(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    if !state_ptr.is_movable {
        return None;
    }
    if !state_ptr.hide_title_bar {
        return None;
    }

    // default handler for resize areas
    let hit = unsafe { DefWindowProcW(handle, msg, wparam, lparam) };
    if matches!(
        hit.0 as u32,
        HTNOWHERE
            | HTRIGHT
            | HTLEFT
            | HTTOPLEFT
            | HTTOP
            | HTTOPRIGHT
            | HTBOTTOMRIGHT
            | HTBOTTOM
            | HTBOTTOMLEFT
    ) {
        return Some(hit.0);
    }

    if state_ptr.state.borrow().is_fullscreen() {
        return Some(HTCLIENT as _);
    }

    let dpi = unsafe { GetDpiForWindow(handle) };
    let frame_y = unsafe { GetSystemMetricsForDpi(SM_CYFRAME, dpi) };

    let mut cursor_point = POINT {
        x: lparam.signed_loword().into(),
        y: lparam.signed_hiword().into(),
    };
    unsafe { ScreenToClient(handle, &mut cursor_point).ok().log_err() };
    if !state_ptr.state.borrow().is_maximized() && cursor_point.y >= 0 && cursor_point.y <= frame_y
    {
        return Some(HTTOP as _);
    }

    let titlebar_rect = state_ptr.state.borrow().get_titlebar_rect();
    if let Ok(titlebar_rect) = titlebar_rect {
        if cursor_point.y < titlebar_rect.bottom {
            let caption_btn_width = (state_ptr.state.borrow().caption_button_width().0
                * state_ptr.state.borrow().scale_factor) as i32;
            if cursor_point.x >= titlebar_rect.right - caption_btn_width {
                return Some(HTCLOSE as _);
            } else if cursor_point.x >= titlebar_rect.right - caption_btn_width * 2 {
                return Some(HTMAXBUTTON as _);
            } else if cursor_point.x >= titlebar_rect.right - caption_btn_width * 3 {
                return Some(HTMINBUTTON as _);
            }

            return Some(HTCAPTION as _);
        }
    }

    Some(HTCLIENT as _)
}

fn handle_nc_mouse_move_msg(
    handle: HWND,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    if !state_ptr.hide_title_bar {
        return None;
    }

    let scale_factor = state_ptr.state.borrow().scale_factor;
    let mut cursor_point = POINT {
        x: lparam.signed_loword().into(),
        y: lparam.signed_hiword().into(),
    };
    unsafe { ScreenToClient(handle, &mut cursor_point).ok().log_err() };
    let event = PlatformInput::MouseMove(MouseMoveEvent {
        position: logical_point(cursor_point.x as f32, cursor_point.y as f32, scale_factor),
        pressed_button: None,
        modifiers: current_modifiers(),
    });
    if with_platform_input_handler(&state_ptr, event).is_some() {
        Some(0)
    } else {
        None
    }
}

fn handle_nc_mouse_down_msg(
    handle: HWND,
    button: MouseButton,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    if !state_ptr.hide_title_bar {
        return None;
    }

    let mut lock = state_ptr.state.borrow_mut();
    let scale_factor = lock.scale_factor;
    let mut cursor_point = POINT {
        x: lparam.signed_loword().into(),
        y: lparam.signed_hiword().into(),
    };
    unsafe { ScreenToClient(handle, &mut cursor_point).ok().log_err() };
    let physical_point = point(DevicePixels(cursor_point.x), DevicePixels(cursor_point.y));
    let click_count = lock.click_state.update(button, physical_point);
    drop(lock);
    let event = PlatformInput::MouseDown(MouseDownEvent {
        button,
        position: logical_point(cursor_point.x as f32, cursor_point.y as f32, scale_factor),
        modifiers: current_modifiers(),
        click_count,
        first_mouse: false,
    });
    if with_platform_input_handler(&state_ptr, event).is_some() {
        return Some(0);
    }

    // Since these are handled in handle_nc_mouse_up_msg we must prevent the default window proc
    if button == MouseButton::Left {
        match wparam.0 as u32 {
            HTMINBUTTON => state_ptr.state.borrow_mut().nc_button_pressed = Some(HTMINBUTTON),
            HTMAXBUTTON => state_ptr.state.borrow_mut().nc_button_pressed = Some(HTMAXBUTTON),
            HTCLOSE => state_ptr.state.borrow_mut().nc_button_pressed = Some(HTCLOSE),
            _ => return None,
        };
        Some(0)
    } else {
        None
    }
}

fn handle_nc_mouse_up_msg(
    handle: HWND,
    button: MouseButton,
    wparam: WPARAM,
    lparam: LPARAM,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    if !state_ptr.hide_title_bar {
        return None;
    }

    let scale_factor = state_ptr.state.borrow().scale_factor;
    let mut cursor_point = POINT {
        x: lparam.signed_loword().into(),
        y: lparam.signed_hiword().into(),
    };
    unsafe { ScreenToClient(handle, &mut cursor_point).ok().log_err() };
    let event = PlatformInput::MouseUp(MouseUpEvent {
        button,
        position: logical_point(cursor_point.x as f32, cursor_point.y as f32, scale_factor),
        modifiers: current_modifiers(),
        click_count: 1,
    });
    if with_platform_input_handler(&state_ptr, event).is_some() {
        return Some(0);
    }

    let last_pressed = state_ptr.state.borrow_mut().nc_button_pressed.take();
    if button == MouseButton::Left && last_pressed.is_some() {
        let last_button = last_pressed.unwrap();
        let mut handled = false;
        match wparam.0 as u32 {
            HTMINBUTTON => {
                if last_button == HTMINBUTTON {
                    unsafe { ShowWindowAsync(handle, SW_MINIMIZE).ok().log_err() };
                    handled = true;
                }
            }
            HTMAXBUTTON => {
                if last_button == HTMAXBUTTON {
                    if state_ptr.state.borrow().is_maximized() {
                        unsafe { ShowWindowAsync(handle, SW_NORMAL).ok().log_err() };
                    } else {
                        unsafe { ShowWindowAsync(handle, SW_MAXIMIZE).ok().log_err() };
                    }
                    handled = true;
                }
            }
            HTCLOSE => {
                if last_button == HTCLOSE {
                    unsafe {
                        PostMessageW(handle, WM_CLOSE, WPARAM::default(), LPARAM::default())
                            .log_err()
                    };
                    handled = true;
                }
            }
            _ => {}
        };
        if handled {
            return Some(0);
        }
    }

    None
}

fn handle_cursor_changed(lparam: LPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    state_ptr.state.borrow_mut().current_cursor = HCURSOR(lparam.0 as _);
    Some(0)
}

fn handle_set_cursor(lparam: LPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    if matches!(
        lparam.loword() as u32,
        HTLEFT | HTRIGHT | HTTOP | HTTOPLEFT | HTTOPRIGHT | HTBOTTOM | HTBOTTOMLEFT | HTBOTTOMRIGHT
    ) {
        return None;
    }
    unsafe { SetCursor(state_ptr.state.borrow().current_cursor) };
    Some(1)
}

fn handle_system_settings_changed(
    handle: HWND,
    state_ptr: Rc<WindowsWindowStatePtr>,
) -> Option<isize> {
    let mut lock = state_ptr.state.borrow_mut();
    let display = lock.display;
    // system settings
    lock.system_settings.update(display);
    // mouse double click
    lock.click_state.system_update();
    // window border offset
    lock.border_offset.update(handle).log_err();
    drop(lock);
    // Force to trigger WM_NCCALCSIZE event to ensure that we handle auto hide
    // taskbar correctly.
    notify_frame_changed(handle);
    Some(0)
}

fn handle_system_command(wparam: WPARAM, state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    if wparam.0 == SC_KEYMENU as usize {
        let mut lock = state_ptr.state.borrow_mut();
        if lock.system_key_handled {
            lock.system_key_handled = false;
            return Some(0);
        }
    }
    None
}

fn handle_system_theme_changed(state_ptr: Rc<WindowsWindowStatePtr>) -> Option<isize> {
    let mut callback = state_ptr
        .state
        .borrow_mut()
        .callbacks
        .appearance_changed
        .take()?;
    callback();
    state_ptr.state.borrow_mut().callbacks.appearance_changed = Some(callback);
    Some(0)
}

fn parse_syskeydown_msg_keystroke(wparam: WPARAM) -> Option<Keystroke> {
    let vk_code = VIRTUAL_KEY(wparam.loword());
    let modifiers = current_modifiers();
    if !modifiers.alt {
        // on Windows, F10 can trigger this event, not just the alt key
        // and we just don't care about F10
        return None;
    }

    Some(Keystroke {
        modifiers,
        key: vk_code.into(),
        ime_key: None,
    })
}

/// Should only return Keydown or modifier events.
fn parse_keydown_msg_to_platform_input(wparam: WPARAM, lparam: LPARAM) -> PlatformInput {
    let vk_code = VIRTUAL_KEY(wparam.loword());
    let modifiers = current_modifiers();

    if is_modifier(vk_code) {
        PlatformInput::ModifiersChanged(ModifiersChangedEvent { modifiers })
    } else {
        PlatformInput::KeyDown(KeyDownEvent {
            keystroke: Keystroke {
                modifiers,
                key: vk_code.into(),
                ime_key: None,
            },
            is_held: lparam.0 & (0x1 << 30) > 0,
        })
    }
}

fn parse_keydup_msg_to_platform_input(wparam: WPARAM) -> PlatformInput {
    let vk_code = VIRTUAL_KEY(wparam.loword());
    let modifiers = current_modifiers();

    if is_modifier(vk_code) {
        PlatformInput::ModifiersChanged(ModifiersChangedEvent { modifiers })
    } else {
        PlatformInput::KeyUp(KeyUpEvent {
            keystroke: Keystroke {
                modifiers,
                key: vk_code.into(),
                ime_key: None,
            },
        })
    }
}

fn parse_char_msg(wparam: WPARAM) -> Option<String> {
    let first_char = char::from_u32((wparam.0 as u16).into())?;
    if first_char.is_control() {
        None
    } else {
        Some(first_char.to_string())
    }
}

fn parse_ime_compostion_string(ctx: HIMC) -> Option<(String, usize)> {
    unsafe {
        let string_len = ImmGetCompositionStringW(ctx, GCS_COMPSTR, None, 0);
        if string_len >= 0 {
            let mut buffer = vec![0u8; string_len as usize + 2];
            ImmGetCompositionStringW(
                ctx,
                GCS_COMPSTR,
                Some(buffer.as_mut_ptr() as _),
                string_len as _,
            );
            let wstring = std::slice::from_raw_parts::<u16>(
                buffer.as_mut_ptr().cast::<u16>(),
                string_len as usize / 2,
            );
            let string = String::from_utf16_lossy(wstring);
            Some((string, string_len as usize / 2))
        } else {
            None
        }
    }
}

#[inline]
fn retrieve_composition_cursor_position(ctx: HIMC) -> usize {
    unsafe { ImmGetCompositionStringW(ctx, GCS_CURSORPOS, None, 0) as usize }
}

fn parse_ime_compostion_result(ctx: HIMC) -> Option<String> {
    unsafe {
        let string_len = ImmGetCompositionStringW(ctx, GCS_RESULTSTR, None, 0);
        if string_len >= 0 {
            let mut buffer = vec![0u8; string_len as usize + 2];
            ImmGetCompositionStringW(
                ctx,
                GCS_RESULTSTR,
                Some(buffer.as_mut_ptr() as _),
                string_len as _,
            );
            let wstring = std::slice::from_raw_parts::<u16>(
                buffer.as_mut_ptr().cast::<u16>(),
                string_len as usize / 2,
            );
            let string = String::from_utf16_lossy(wstring);
            Some(string)
        } else {
            None
        }
    }
}

#[inline]
fn is_virtual_key_pressed(vkey: VIRTUAL_KEY) -> bool {
    unsafe { GetKeyState(vkey.0 as i32) < 0 }
}

fn is_modifier(virtual_key: VIRTUAL_KEY) -> bool {
    matches!(
        virtual_key,
        VK_CONTROL | VK_MENU | VK_SHIFT | VK_LWIN | VK_RWIN
    )
}

#[inline]
pub(crate) fn current_modifiers() -> Modifiers {
    Modifiers {
        control: is_virtual_key_pressed(VK_CONTROL),
        alt: is_virtual_key_pressed(VK_MENU),
        shift: is_virtual_key_pressed(VK_SHIFT),
        platform: is_virtual_key_pressed(VK_LWIN) || is_virtual_key_pressed(VK_RWIN),
        function: false,
    }
}

fn get_client_area_insets(
    handle: HWND,
    is_maximized: bool,
    windows_version: WindowsVersion,
) -> RECT {
    // For maximized windows, Windows outdents the window rect from the screen's client rect
    // by `frame_thickness` on each edge, meaning `insets` must contain `frame_thickness`
    // on all sides (including the top) to avoid the client area extending onto adjacent
    // monitors.
    //
    // For non-maximized windows, things become complicated:
    //
    // - On Windows 10
    // The top inset must be zero, since if there is any nonclient area, Windows will draw
    // a full native titlebar outside the client area. (This doesn't occur in the maximized
    // case.)
    //
    // - On Windows 11
    // The top inset is calculated using an empirical formula that I derived through various
    // tests. Without this, the top 1-2 rows of pixels in our window would be obscured.
    let dpi = unsafe { GetDpiForWindow(handle) };
    let frame_thickness = get_frame_thickness(dpi);
    let top_insets = if is_maximized {
        frame_thickness
    } else {
        match windows_version {
            WindowsVersion::Win10 => 0,
            WindowsVersion::Win11 => (dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32).round() as i32,
        }
    };
    RECT {
        left: frame_thickness,
        top: top_insets,
        right: frame_thickness,
        bottom: frame_thickness,
    }
}

// there is some additional non-visible space when talking about window
// borders on Windows:
// - SM_CXSIZEFRAME: The resize handle.
// - SM_CXPADDEDBORDER: Additional border space that isn't part of the resize handle.
fn get_frame_thickness(dpi: u32) -> i32 {
    let resize_frame_thickness = unsafe { GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi) };
    let padding_thickness = unsafe { GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi) };
    resize_frame_thickness + padding_thickness
}

fn notify_frame_changed(handle: HWND) {
    unsafe {
        SetWindowPos(
            handle,
            None,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED
                | SWP_NOACTIVATE
                | SWP_NOCOPYBITS
                | SWP_NOMOVE
                | SWP_NOOWNERZORDER
                | SWP_NOREPOSITION
                | SWP_NOSENDCHANGING
                | SWP_NOSIZE
                | SWP_NOZORDER,
        )
        .log_err();
    }
}

fn with_input_handler<F, R>(state_ptr: &Rc<WindowsWindowStatePtr>, f: F) -> Option<R>
where
    F: FnOnce(&mut PlatformInputHandler) -> R,
{
    let mut input_handler = state_ptr.state.borrow_mut().input_handler.take()?;
    let result = f(&mut input_handler);
    state_ptr.state.borrow_mut().input_handler = Some(input_handler);
    Some(result)
}

fn with_input_handler_and_scale_factor<F, R>(
    state_ptr: &Rc<WindowsWindowStatePtr>,
    f: F,
) -> Option<R>
where
    F: FnOnce(&mut PlatformInputHandler, f32) -> Option<R>,
{
    let mut lock = state_ptr.state.borrow_mut();
    let mut input_handler = lock.input_handler.take()?;
    let scale_factor = lock.scale_factor;
    drop(lock);
    let result = f(&mut input_handler, scale_factor);
    state_ptr.state.borrow_mut().input_handler = Some(input_handler);
    result
}

fn with_keyboard_input_handler<F>(
    state_ptr: &Rc<WindowsWindowStatePtr>,
    event: PlatformInput,
    f: F,
) -> Option<()>
where
    F: FnOnce(&mut RefMut<'_, WindowsWindowState>, bool),
{
    // let mut lock = state_ptr.state.borrow_mut();
    // let mut handler = lock.callbacks.input.take()?;
    // drop(lock);
    let mut handler = state_ptr.state.borrow_mut().callbacks.input.take()?;
    let handled = !handler(event).propagate;
    let mut lock = state_ptr.state.borrow_mut();
    lock.callbacks.input = Some(handler);
    f(&mut lock, handled);
    if handled {
        Some(())
    } else {
        None
    }
}

fn with_platform_input_handler(
    state_ptr: &Rc<WindowsWindowStatePtr>,
    event: PlatformInput,
) -> Option<()> {
    let mut handler = state_ptr.state.borrow_mut().callbacks.input.take()?;
    let handled = !handler(event).propagate;
    state_ptr.state.borrow_mut().callbacks.input = Some(handler);
    if handled {
        Some(())
    } else {
        None
    }
}

impl From<VIRTUAL_KEY> for VirtualKeyCode {
    fn from(value: VIRTUAL_KEY) -> Self {
        match value {
            // VirtualKeyCode::Unknown => todo!(),
            // VirtualKeyCode::Function => todo!(),
            VK_CANCEL => VirtualKeyCode::Cancel,
            VK_BACK => VirtualKeyCode::Backspace,
            VK_TAB => VirtualKeyCode::Tab,
            VK_CLEAR => VirtualKeyCode::Clear,
            VK_RETURN => VirtualKeyCode::Enter,
            VK_SHIFT => VirtualKeyCode::Shift,
            VK_CONTROL => VirtualKeyCode::Control,
            VK_MENU => VirtualKeyCode::Alt,
            VK_PAUSE => VirtualKeyCode::Pause,
            VK_CAPITAL => VirtualKeyCode::Capital,
            VK_KANA => VirtualKeyCode::Kana,
            // VK_HANGUL => VirtualKeyCode::Hangul,
            VK_JUNJA => VirtualKeyCode::Junja,
            VK_FINAL => VirtualKeyCode::Final,
            VK_HANJA => VirtualKeyCode::Hanja,
            // VK_KANJI => VirtualKeyCode::Kanji,
            VK_ESCAPE => VirtualKeyCode::Escape,
            VK_CONVERT => VirtualKeyCode::Convert,
            VK_NONCONVERT => VirtualKeyCode::Nonconvert,
            VK_ACCEPT => VirtualKeyCode::Accept,
            VK_MODECHANGE => VirtualKeyCode::ModeChange,
            VK_SPACE => VirtualKeyCode::Space,
            VK_PRIOR => VirtualKeyCode::PageUp,
            VK_NEXT => VirtualKeyCode::PageDown,
            VK_END => VirtualKeyCode::End,
            VK_HOME => VirtualKeyCode::Home,
            VK_LEFT => VirtualKeyCode::Left,
            VK_UP => VirtualKeyCode::Up,
            VK_RIGHT => VirtualKeyCode::Right,
            VK_DOWN => VirtualKeyCode::Down,
            VK_SELECT => VirtualKeyCode::Select,
            VK_PRINT => VirtualKeyCode::Print,
            VK_EXECUTE => VirtualKeyCode::Execute,
            VK_SNAPSHOT => VirtualKeyCode::PrintScreen,
            VK_INSERT => VirtualKeyCode::Insert,
            VK_DELETE => VirtualKeyCode::Delete,
            VK_HELP => VirtualKeyCode::Help,
            VK_0 => VirtualKeyCode::Digital0,
            VK_1 => VirtualKeyCode::Digital1,
            VK_2 => VirtualKeyCode::Digital2,
            VK_3 => VirtualKeyCode::Digital3,
            VK_4 => VirtualKeyCode::Digital4,
            VK_5 => VirtualKeyCode::Digital5,
            VK_6 => VirtualKeyCode::Digital6,
            VK_7 => VirtualKeyCode::Digital7,
            VK_8 => VirtualKeyCode::Digital8,
            VK_9 => VirtualKeyCode::Digital9,
            VK_A => VirtualKeyCode::A,
            VK_B => VirtualKeyCode::B,
            VK_C => VirtualKeyCode::C,
            VK_D => VirtualKeyCode::D,
            VK_E => VirtualKeyCode::E,
            VIRTUAL_KEY(70u16) => VirtualKeyCode::F,
            VK_G => VirtualKeyCode::G,
            VK_H => VirtualKeyCode::H,
            VK_I => VirtualKeyCode::I,
            VK_J => VirtualKeyCode::J,
            VK_K => VirtualKeyCode::K,
            VK_L => VirtualKeyCode::L,
            VK_M => VirtualKeyCode::M,
            VK_N => VirtualKeyCode::N,
            VK_O => VirtualKeyCode::O,
            VK_P => VirtualKeyCode::P,
            VK_Q => VirtualKeyCode::Q,
            VK_R => VirtualKeyCode::R,
            VK_S => VirtualKeyCode::S,
            VK_T => VirtualKeyCode::T,
            VK_U => VirtualKeyCode::U,
            VK_V => VirtualKeyCode::V,
            VK_W => VirtualKeyCode::W,
            VK_X => VirtualKeyCode::X,
            VK_Y => VirtualKeyCode::Y,
            VK_Z => VirtualKeyCode::Z,
            VK_LWIN => VirtualKeyCode::LeftPlatform,
            VK_RWIN => VirtualKeyCode::RightPlatform,
            VK_APPS => VirtualKeyCode::App,
            VK_SLEEP => VirtualKeyCode::Sleep,
            VK_NUMPAD0 => VirtualKeyCode::Numpad0,
            VK_NUMPAD1 => VirtualKeyCode::Numpad1,
            VK_NUMPAD2 => VirtualKeyCode::Numpad2,
            VK_NUMPAD3 => VirtualKeyCode::Numpad3,
            VK_NUMPAD4 => VirtualKeyCode::Numpad4,
            VK_NUMPAD5 => VirtualKeyCode::Numpad5,
            VK_NUMPAD6 => VirtualKeyCode::Numpad6,
            VK_NUMPAD7 => VirtualKeyCode::Numpad7,
            VK_NUMPAD8 => VirtualKeyCode::Numpad8,
            VK_NUMPAD9 => VirtualKeyCode::Numpad9,
            VK_MULTIPLY => VirtualKeyCode::Multiply,
            VK_ADD => VirtualKeyCode::Add,
            VK_SEPARATOR => VirtualKeyCode::Separator,
            VK_SUBTRACT => VirtualKeyCode::Subtract,
            VK_DECIMAL => VirtualKeyCode::Decimal,
            VK_DIVIDE => VirtualKeyCode::Divide,
            VK_F1 => VirtualKeyCode::F1,
            VK_F2 => VirtualKeyCode::F2,
            VK_F3 => VirtualKeyCode::F3,
            VK_F4 => VirtualKeyCode::F4,
            VK_F5 => VirtualKeyCode::F5,
            VK_F6 => VirtualKeyCode::F6,
            VK_F7 => VirtualKeyCode::F7,
            VK_F8 => VirtualKeyCode::F8,
            VK_F9 => VirtualKeyCode::F9,
            VK_F10 => VirtualKeyCode::F10,
            VK_F11 => VirtualKeyCode::F11,
            VK_F12 => VirtualKeyCode::F12,
            VK_F13 => VirtualKeyCode::F13,
            VK_F14 => VirtualKeyCode::F14,
            VK_F15 => VirtualKeyCode::F15,
            VK_F16 => VirtualKeyCode::F16,
            VK_F17 => VirtualKeyCode::F17,
            VK_F18 => VirtualKeyCode::F18,
            VK_F19 => VirtualKeyCode::F19,
            VK_F20 => VirtualKeyCode::F20,
            VK_F21 => VirtualKeyCode::F21,
            VK_F22 => VirtualKeyCode::F22,
            VK_F23 => VirtualKeyCode::F23,
            VK_F24 => VirtualKeyCode::F24,
            VK_NUMLOCK => VirtualKeyCode::NumLock,
            VK_SCROLL => VirtualKeyCode::ScrollLock,
            VK_LSHIFT => VirtualKeyCode::LeftShift,
            VK_RSHIFT => VirtualKeyCode::RightShift,
            VK_LCONTROL => VirtualKeyCode::LeftControl,
            VK_RCONTROL => VirtualKeyCode::RightControl,
            VK_LMENU => VirtualKeyCode::LeftAlt,
            VK_RMENU => VirtualKeyCode::RightAlt,
            VK_BROWSER_BACK => VirtualKeyCode::BrowserBack,
            VK_BROWSER_FORWARD => VirtualKeyCode::BrowserForward,
            VK_BROWSER_REFRESH => VirtualKeyCode::BrowserRefresh,
            VK_BROWSER_STOP => VirtualKeyCode::BrowserStop,
            VK_BROWSER_SEARCH => VirtualKeyCode::BrowserSearch,
            VK_BROWSER_FAVORITES => VirtualKeyCode::BrowserFavorites,
            VK_BROWSER_HOME => VirtualKeyCode::BrowserHome,
            VK_VOLUME_MUTE => VirtualKeyCode::VolumeMute,
            VK_VOLUME_DOWN => VirtualKeyCode::VolumeDown,
            VK_VOLUME_UP => VirtualKeyCode::VolumeUp,
            VK_MEDIA_NEXT_TRACK => VirtualKeyCode::MediaNextTrack,
            VK_MEDIA_PREV_TRACK => VirtualKeyCode::MediaPrevTrack,
            VK_MEDIA_STOP => VirtualKeyCode::MediaStop,
            VK_MEDIA_PLAY_PAUSE => VirtualKeyCode::MediaPlayPause,
            VK_LAUNCH_MAIL => VirtualKeyCode::LaunchMail,
            VK_LAUNCH_MEDIA_SELECT => VirtualKeyCode::LaunchMediaSelect,
            VK_LAUNCH_APP1 => VirtualKeyCode::LaunchApp1,
            VK_LAUNCH_APP2 => VirtualKeyCode::LaunchApp2,
            VK_OEM_1 => VirtualKeyCode::OEM1,
            VK_OEM_PLUS => VirtualKeyCode::OEMPlus,
            VK_OEM_COMMA => VirtualKeyCode::OEMComma,
            VK_OEM_MINUS => VirtualKeyCode::OEMMinus,
            VK_OEM_PERIOD => VirtualKeyCode::OEMPeriod,
            VK_OEM_2 => VirtualKeyCode::OEM2,
            VK_OEM_3 => VirtualKeyCode::OEM3,
            VK_OEM_4 => VirtualKeyCode::OEM4,
            VK_OEM_5 => VirtualKeyCode::OEM5,
            VK_OEM_6 => VirtualKeyCode::OEM6,
            VK_OEM_7 => VirtualKeyCode::OEM7,
            VK_OEM_8 => VirtualKeyCode::OEM8,
            VK_OEM_102 => VirtualKeyCode::OEM102,
            VK_PROCESSKEY => VirtualKeyCode::ProcessKey,
            VK_PACKET => VirtualKeyCode::Packet,
            VK_ATTN => VirtualKeyCode::Attn,
            VK_CRSEL => VirtualKeyCode::CrSel,
            VK_EXSEL => VirtualKeyCode::ExSel,
            VK_EREOF => VirtualKeyCode::EraseEOF,
            VK_PLAY => VirtualKeyCode::Play,
            VK_ZOOM => VirtualKeyCode::Zoom,
            VK_PA1 => VirtualKeyCode::PA1,
            VK_OEM_CLEAR => VirtualKeyCode::OEMClear,
            _ => VirtualKeyCode::Unknown,
        }
    }
}
