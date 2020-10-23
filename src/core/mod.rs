extern crate winapi;

use winapi::{ um::{winuser, dwmapi, winnt}, shared::{windef, minwindef, winerror}, ctypes };
use crate::tile;

#[derive(Copy, Debug, Clone, PartialEq)]
pub struct Window {
    hwnd: windef::HWND
}

// :(, There doesn't appear to be a way to pass context/data into SetWinEvenHook or
// SetWinEvenHookEx, and I don't know enough about Rust or the Windows API to know what
// the optimal way of retrieving this data should be. Hopefully, when I find out it won't
// be expensive to fix... 
static mut WINDOWS: Vec<Window> = vec![];
static mut WIN_CLOSED: Vec<Window> = vec![];
static mut FOCUSED_WINDOW: Option<Window> = None;
static mut PREV_WINDOW: Option<Window> = None;

pub fn run() -> Result<i32, std::io::Error> {
    let init_windows = get_initial_windows();
    let win_dimensions = get_window_dimensions();

    let root = tile_existing_windows(init_windows, win_dimensions);
    redraw_nodes(&root);

    hook_and_loop(root);

    Ok(0)
}

fn hook_and_loop(mut root: tile::Node<Window>) {
    unsafe {
        winuser::SetWinEventHook(winuser::EVENT_OBJECT_CREATE, winuser::EVENT_OBJECT_DESTROY,
            std::ptr::null_mut(), Some(window_event_hook), 0, 0, winuser::WINEVENT_OUTOFCONTEXT);

        winuser::SetWinEventHook(winuser::EVENT_OBJECT_FOCUS, winuser::EVENT_OBJECT_FOCUS,
            std::ptr::null_mut(), Some(focus_changed), 0, 0, winuser::WINEVENT_OUTOFCONTEXT);

       let mut msg: winuser::MSG = Default::default();

       loop {
           let msg_exists = winuser::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, winuser::PM_REMOVE);
           if msg_exists == minwindef::TRUE {
               winuser::DispatchMessageW(&mut msg);
           }

           while !WINDOWS.is_empty() {
               // instead of calling redrawnodes every time, have tile::tile return a reference to
               // the seperator so it could just redraw those?
               if let Some(win) = PREV_WINDOW {
                   let focused_node = tile::find_node::<Window>(&mut root, win);
                   if let Some(last_focused) = focused_node {
                       tile::tile::<Window>(last_focused, tile::Orientation::Vertical, WINDOWS.remove(0));
                   } else {
                       tile::tile::<Window>(&mut root, tile::Orientation::Vertical, WINDOWS.remove(0));
                   }
               } else {
                   tile::tile::<Window>(&mut root, tile::Orientation::Vertical, WINDOWS.remove(0));
               }
               redraw_nodes(&root);
           }

           while !WIN_CLOSED.is_empty() {
               let window = WIN_CLOSED.remove(0);
               tile::untile::<Window>(&mut root, &window);
               redraw_nodes(&root);
           }
       }
    }
}

fn tile_existing_windows(mut windows: Vec<Window>, dim: tile::Dimensions) -> tile::Node<Window> {
    let mut root: tile::Node<Window> = tile::Node {
        node_type: tile::NodeType::Empty,
        dim: dim
    };

    while !windows.is_empty() {
        tile::tile::<Window>(&mut root, tile::Orientation::Horizontal, windows.remove(0));
    }

    root
}

fn redraw_nodes(root: &tile::Node<Window>) {
    match &root.node_type {
        tile::NodeType::Separator(_, left_child, right_child) => {
            redraw_nodes(left_child);
            redraw_nodes(right_child);
        },
        tile::NodeType::Window(win) => {
            show_window(win.hwnd);
            set_window_pos(win.hwnd, root.dim.x.0, root.dim.y.0,
                root.dim.x.1, root.dim.y.1);
        },
        tile::NodeType::Empty => return
    }
}

fn get_initial_windows() -> Vec<Window> {
    let win_handles: Vec<Window> = Vec::new();
    unsafe {
        let res = winuser::EnumWindows(Some(enum_windows), &win_handles as *const _ as minwindef::LPARAM);
        if res == minwindef::FALSE {
            // TODO consider not failing in future, instead just continue
            panic!("Could not retrieve windows");
        }
    }
    win_handles
}

fn get_window_dimensions() -> tile::Dimensions {
    let monitor: windef::HMONITOR;
    unsafe {
        monitor = get_primary_monitor();
    }
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
        y: (monitor_info.rcMonitor.top, monitor_info.rcMonitor.bottom)
    }
}

fn set_window_pos(hwnd: windef::HWND, x: i32, y: i32, cx: i32, cy: i32) -> bool {
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

fn show_window(hwnd: windef::HWND) {
    unsafe {
        winuser::ShowWindow(hwnd, winuser::SW_RESTORE);
    }
}

// TODO does this need to be a separate function?
unsafe extern "system"
fn get_primary_monitor() -> windef::HMONITOR {
    let point = windef::POINT { x:0, y:0 };
    winuser::MonitorFromPoint(point, winuser::MONITOR_DEFAULTTOPRIMARY)
}

unsafe extern "system"
fn enum_windows(hwnd: windef::HWND, l_param: minwindef::LPARAM) -> minwindef::BOOL {
    let win_title_len = winuser::GetWindowTextLengthW(hwnd) + 1;
    let mut win_title_vec: Vec<u16> = Vec::with_capacity(win_title_len as usize);
    let res_len = winuser::GetWindowTextW(hwnd, win_title_vec.as_mut_ptr(), win_title_len);

    // hide windows with no title and don't show up on screen,
    // since windows seems to have a lot of those
    if res_len > 0 && winuser::IsWindowVisible(hwnd) == minwindef::TRUE {
        // also hide any windows that are 'cloaked', that also don't
        // show up on screen
        let mut cloaked = 0u32;
        let result = dwmapi::DwmGetWindowAttribute(hwnd, dwmapi::DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut ctypes::c_void, std::mem::size_of::<u32>() as u32);
        if result != winerror::S_OK {
            cloaked = 0;
        }

        if cloaked != dwmapi::DWM_CLOAKED_SHELL && cloaked != dwmapi::DWM_CLOAKED_APP && cloaked != dwmapi::DWM_CLOAKED_INHERITED {
            win_title_vec.set_len((res_len) as usize);
            let window_name = String::from_utf16_lossy(&win_title_vec);
            if !window_name.contains("NVIDIA GeForce Overlay") && !window_name.contains("Program Manager") {
                let handles_ptr = l_param as *mut Vec<Window>;
                let handles: &mut Vec<Window> = &mut *handles_ptr;
                handles.push(Window { hwnd });
            }
        }
    }
    minwindef::TRUE
}

unsafe extern "system"
fn window_event_hook(_event_hook: windef::HWINEVENTHOOK, event: minwindef::DWORD, hwnd: windef::HWND, id_obj: winnt::LONG, id_child: winnt::LONG, _id_event_thread: minwindef::DWORD, _time: minwindef::DWORD) {
    use winapi::um::winuser::{ EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, OBJID_WINDOW, INDEXID_CONTAINER, WS_BORDER, GetClientRect, GetWindowTextLengthW, GetWindowTextW, GetWindowLongW, GWL_STYLE };

    // TODO this is a very aggressive filter, different parts were plucked from different
    // suggestions online about how to get random hidden windows from appearing. Find a better way
    // to do this in the future, hopefully.
    if !((event == EVENT_OBJECT_CREATE || event == EVENT_OBJECT_DESTROY) && id_obj == OBJID_WINDOW && id_child == INDEXID_CONTAINER) {
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
        WINDOWS.push(Window { hwnd });
    }

    if event == EVENT_OBJECT_DESTROY {
        WIN_CLOSED.push(Window { hwnd });
    }
}

unsafe extern "system"
fn focus_changed(_event_hook: windef::HWINEVENTHOOK, _event: minwindef::DWORD, hwnd: windef::HWND, _id_obj: winnt::LONG, _id_child: winnt::LONG, _id_event_thread: minwindef::DWORD, _time: minwindef::DWORD) {
    PREV_WINDOW = FOCUSED_WINDOW;
    FOCUSED_WINDOW = Some(Window { hwnd });
}

