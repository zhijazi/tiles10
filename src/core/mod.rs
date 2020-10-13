extern crate winapi;

use winapi::{ um::{winuser, dwmapi}, shared::{windef, minwindef, winerror}, ctypes };
use crate::tile;

#[derive(Debug, Clone)]
pub struct Window {
    hwnd: windef::HWND
}

pub fn run() -> Result<i32, std::io::Error> {
    let open_windows = get_initial_windows();
    let win_dimensions = get_window_dimensions();

    let root = tile_existing_windows(open_windows, win_dimensions);
    redraw_nodes(&root);

    // event loop

    Ok(0)
}

fn tile_existing_windows(mut windows: Vec<Window>, dim: tile::Dimensions) -> tile::Node<Window> {
    let mut root: tile::Node<Window> = tile::Node {
        node_type: tile::NodeType::Empty,
        dim: dim,
        left: None,
        right: None
    };

    while !windows.is_empty() {
        tile::tile::<Window>(&mut root, tile::Orientation::Horizontal, windows.remove(0));
    }

    root
}

fn redraw_nodes(root: &tile::Node<Window>) {
    match &root.node_type {
        tile::NodeType::Separator(_) => {
            if let Some(win) = root.left.as_ref() {
                redraw_nodes(win);
            }
            if let Some(win) = root.right.as_ref() {
                redraw_nodes(win);
            }
        },
        tile::NodeType::Window(win) => {
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
            if !window_name.contains("NVIDIA GeForce Overlay") && !window_name.contains("Program Manager") && !window_name.contains("Windows PowerShell") {
                println!("{}", window_name);
                let handles_ptr = l_param as *mut Vec<Window>;
                let handles: &mut Vec<Window> = &mut *handles_ptr;
                handles.push(Window { hwnd });
            }
        }
    }
    minwindef::TRUE
}

