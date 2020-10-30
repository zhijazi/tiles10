use crate::tile;
use winapi::{
    ctypes,
    shared::{minwindef, windef, winerror},
    um::{dwmapi, winnt, winuser},
};

pub enum WindowEvent {
    Created(windef::HWND),
    Destroyed(windef::HWND),
    FocusChanged(windef::HWND),
    OrientationChanged
}

pub static mut WIN_EVENT: Option<WindowEvent> = None;

pub fn create_hooks() {
    let window_hook_res: windef::HWINEVENTHOOK;
    let focus_hook_res: windef::HWINEVENTHOOK;
    unsafe {
        window_hook_res = winuser::SetWinEventHook(
            winuser::EVENT_OBJECT_CREATE,
            winuser::EVENT_OBJECT_DESTROY,
            std::ptr::null_mut(),
            Some(window_event_hook),
            0,
            0,
            winuser::WINEVENT_OUTOFCONTEXT,
        );

        if let None = window_hook_res.as_ref() {
            panic!("Could not set window create & window delete hooks. Aborting");
        }

        focus_hook_res = winuser::SetWinEventHook(
            winuser::EVENT_OBJECT_FOCUS,
            winuser::EVENT_OBJECT_FOCUS,
            std::ptr::null_mut(),
            Some(focus_changed),
            0,
            0,
            winuser::WINEVENT_OUTOFCONTEXT,
        );

        if let None = focus_hook_res.as_ref() {
            panic!("Could not set focus changed hook. Aborting");
        }

        if winuser::RegisterHotKey(std::ptr::null_mut(), 0, winuser::MOD_ALT as minwindef::UINT, 0x58) == minwindef::FALSE {
            panic!("Could not register hot key");
        }


        if winuser::RegisterHotKey(std::ptr::null_mut(), 0, winuser::MOD_ALT as minwindef::UINT, 0x43) == minwindef::FALSE {
            panic!("Could not register hot key");
        }
    }
}

pub fn send_message() -> Option<WindowEvent> {
    let mut msg: winuser::MSG = Default::default();
    unsafe {
        let msg_exists =
            winuser::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, winuser::PM_REMOVE);
        if msg_exists == minwindef::TRUE {
            match msg.message {
                winuser::WM_HOTKEY => { WIN_EVENT = Some(WindowEvent::OrientationChanged) }
                _ => { }
            }

            winuser::TranslateMessage(&msg);
            winuser::DispatchMessageW(&mut msg);
        }
        WIN_EVENT.take()
    }
}

pub fn get_active_window() -> windef::HWND {
    unsafe { winuser::GetActiveWindow() }
}

pub fn get_initial_windows() -> Vec<windef::HWND> {
    let win_handles: Vec<windef::HWND> = Vec::new();
    unsafe {
        let res = winuser::EnumWindows(
            Some(enum_windows),
            &win_handles as *const _ as minwindef::LPARAM,
        );
        if res == minwindef::FALSE {
            // TODO consider not failing in future, instead just continue
            panic!("Could not retrieve windows");
        }
    }
    win_handles
}

pub fn set_window_pos(hwnd: windef::HWND, x: i32, y: i32, cx: i32, cy: i32) -> bool {
    let set_pos_res: minwindef::BOOL;
    unsafe {
        // TODO Removed HWND_TOPMOST during development... evaluate if it we want window
        // to actually be TOPMOST, since it could be annoying once the WM is closed
        set_pos_res = winuser::SetWindowPos(hwnd, winuser::HWND_TOP, x, y, cx, cy, 0u32);
    }

    if set_pos_res == minwindef::FALSE {
        return false;
    }

    true
}

pub fn get_window_dimensions() -> tile::Dimensions {
    let monitor: windef::HMONITOR;
    monitor = get_primary_monitor();
    let mut monitor_info = winuser::MONITORINFO {
        cbSize: std::mem::size_of::<winuser::MONITORINFO>() as _,
        ..Default::default()
    };

    let monitor_info_res: minwindef::BOOL;
    unsafe {
        monitor_info_res = winuser::GetMonitorInfoW(monitor, &mut monitor_info);
    }

    if monitor_info_res == minwindef::FALSE {
        panic!("Could not retrieve monitor information.");
    }

    tile::Dimensions {
        x: (monitor_info.rcMonitor.left, monitor_info.rcMonitor.right),
        y: (monitor_info.rcMonitor.top, monitor_info.rcMonitor.bottom),
    }
}

pub fn show_window(hwnd: windef::HWND) {
    unsafe {
        winuser::ShowWindow(hwnd, winuser::SW_RESTORE);
    }
}

unsafe extern "system" fn focus_changed(
    _event_hook: windef::HWINEVENTHOOK,
    _event: minwindef::DWORD,
    hwnd: windef::HWND,
    _id_obj: winnt::LONG,
    _id_child: winnt::LONG,
    _id_event_thread: minwindef::DWORD,
    _time: minwindef::DWORD,
) {
    WIN_EVENT = Some(WindowEvent::FocusChanged(hwnd));
}

unsafe extern "system" fn window_event_hook(
    _event_hook: windef::HWINEVENTHOOK,
    event: minwindef::DWORD,
    hwnd: windef::HWND,
    id_obj: winnt::LONG,
    id_child: winnt::LONG,
    _id_event_thread: minwindef::DWORD,
    _time: minwindef::DWORD,
) {
    use winapi::um::winuser::{
        GetClientRect, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, EVENT_OBJECT_CREATE,
        EVENT_OBJECT_DESTROY, GWL_STYLE, INDEXID_CONTAINER, OBJID_WINDOW, WS_BORDER,
    };

    // TODO this is a very aggressive filter, different parts were plucked from different
    // suggestions online about how to get random hidden windows from appearing. Find a better way
    // to do this in the future, hopefully.
    if !((event == EVENT_OBJECT_CREATE || event == EVENT_OBJECT_DESTROY)
        && id_obj == OBJID_WINDOW
        && id_child == INDEXID_CONTAINER)
    {
        return;
    }

    let win_len = GetWindowTextLengthW(hwnd) + 1;
    if win_len - 1 == 0 {
        return;
    }

    let mut v: Vec<u16> = Vec::with_capacity(win_len as usize);
    let read_len = GetWindowTextW(hwnd, v.as_mut_ptr(), win_len);
    if read_len == 0 {
        return;
    }

    let lstyle = GetWindowLongW(hwnd, GWL_STYLE);
    if lstyle == 0 {
        return;
    }

    let mut r_area: windef::RECT = Default::default();
    if GetClientRect(hwnd, &mut r_area) == minwindef::FALSE {
        return;
    }

    if (lstyle & (WS_BORDER as i32)) != WS_BORDER as i32 {
        return;
    }

    if event == EVENT_OBJECT_CREATE {
        WIN_EVENT = Some(WindowEvent::Created(hwnd));
    }

    if event == EVENT_OBJECT_DESTROY {
        WIN_EVENT = Some(WindowEvent::Destroyed(hwnd));
    }
}

// TODO does this need to be a separate function?
pub fn get_primary_monitor() -> windef::HMONITOR {
    let point = windef::POINT { x: 0, y: 0 };
    unsafe { winuser::MonitorFromPoint(point, winuser::MONITOR_DEFAULTTOPRIMARY) }
}

unsafe extern "system" fn enum_windows(
    hwnd: windef::HWND,
    l_param: minwindef::LPARAM,
) -> minwindef::BOOL {
    let win_title_len = winuser::GetWindowTextLengthW(hwnd) + 1;
    let mut win_title_vec: Vec<u16> = Vec::with_capacity(win_title_len as usize);
    let res_len = winuser::GetWindowTextW(hwnd, win_title_vec.as_mut_ptr(), win_title_len);

    // hide windows with no title and don't show up on screen,
    // since windows seems to have a lot of those
    if res_len > 0 && winuser::IsWindowVisible(hwnd) == minwindef::TRUE {
        // also hide any windows that are 'cloaked', that also don't
        // show up on screen
        let mut cloaked = 0u32;
        let result = dwmapi::DwmGetWindowAttribute(
            hwnd,
            dwmapi::DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut ctypes::c_void,
            std::mem::size_of::<u32>() as u32,
        );
        if result != winerror::S_OK {
            cloaked = 0;
        }

        if cloaked != dwmapi::DWM_CLOAKED_SHELL
            && cloaked != dwmapi::DWM_CLOAKED_APP
            && cloaked != dwmapi::DWM_CLOAKED_INHERITED
        {
            win_title_vec.set_len((res_len) as usize);
            let window_name = String::from_utf16_lossy(&win_title_vec);
            if !window_name.contains("NVIDIA GeForce Overlay")
                && !window_name.contains("Program Manager")
            {
                let handles_ptr = l_param as *mut Vec<windef::HWND>;
                let handles: &mut Vec<windef::HWND> = &mut *handles_ptr;
                handles.push(hwnd);
            }
        }
    }
    minwindef::TRUE
}
