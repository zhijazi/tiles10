extern crate winapi;

use winapi::{ um::{winuser, dwmapi}, shared::{windef, minwindef, winerror}, ctypes };
use crate::tile;

#[derive(Clone)]
pub struct Window {
    hwnd: windef::HWND
}

pub fn run() -> Result<i32, std::io::Error> {
    let mut open_windows = get_initial_windows();
    let win_dimensions = get_window_dimensions();
    //set_window_pos(open_windows[0], win_dimensions.x.0, win_dimensions.y.0,
     //   win_dimensions.x.1, win_dimensions.y.1);

    // generate initial windows. TODO move initial generation into self-contained
    // function
    Ok(0)
}

fn resize_children(root: tile::Node<Window>) {
    // do a match (root.Nodetype) here and check:
    // if horizontal -> split horizontal and propgate down
    // if vertical -> split vertical and propogate down
    // if window -> resize to new dimensions and stop
}

fn redraw_nodes(root: tile::Node<Window>) {
    match root.node_type {
        tile::NodeType::Separator(_) => {
            redraw_nodes(root.left.unwrap());
            redraw_nodes(root.right.unwrap());
        },
        tile::NodeType::Window(win) => {
            set_window_pos(win.hwnd, root.dim.x.0, root.dim.y.0,
                root.dim.x.1, root.dim.y.1);
        }
    }
}

fn get_initial_windows() -> Vec<windef::HWND> {
    let win_handles: Vec<windef::HWND> = Vec::new();
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
                let handles_ptr = l_param as *mut Vec<windef::HWND>;
                let handles: &mut Vec<windef::HWND> = &mut *handles_ptr;
                handles.push(hwnd);
            }
        }
    }
    minwindef::TRUE
}
