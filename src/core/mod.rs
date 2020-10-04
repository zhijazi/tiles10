extern crate winapi;

use winapi::{ um::{winuser, dwmapi}, shared::{windef, minwindef, winerror}, ctypes };

pub fn run() -> Result<i32, std::io::Error> {
    get_initial_windows();
    Ok(0)
}

fn get_initial_windows() -> Vec<windef::HWND> {
    unsafe {
        let res = winuser::EnumWindows(Some(enum_windows), 0isize);
        if res == minwindef::FALSE {
            // TODO consider not failing in future, instead just continue
            panic!("Could not retrieve windows");
        }
    }
    vec!()
}

unsafe extern "system" fn enum_windows(hwnd: windef::HWND, _: minwindef::LPARAM) -> minwindef::BOOL {
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
                println!("{}", window_name);
            }
        }
    }

    return minwindef::TRUE;
}
