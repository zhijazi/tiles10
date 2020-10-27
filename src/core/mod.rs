extern crate winapi;

use crate::internal;
use crate::tile;
use winapi::{shared::{windef}};

pub fn run() -> Result<i32, std::io::Error> {
    let init_windows = internal::get_initial_windows();
    let win_dimensions = internal::get_window_dimensions();

    let root = tile_existing_windows(init_windows, win_dimensions);
    redraw_nodes(&root);

    hook_and_loop(root);

    Ok(0)
}

fn hook_and_loop(mut root: tile::Node<windef::HWND>) {
    internal::create_hooks();
    let mut prev_window = internal::get_active_window();
    let mut current_focus = prev_window.clone();
    loop {
        if let Some(event) = internal::send_message() {
            match event {
                internal::WindowEvent::Created(window) => {
                    tile_new_window(&mut root, window, prev_window);
                }
                internal::WindowEvent::Destroyed(window) => {
                    untile_window(&mut root, window);
                }
                internal::WindowEvent::FocusChanged(window) => {
                    prev_window = current_focus;
                    current_focus = window;
                }
            }
        }
    }
}

fn tile_new_window(
    mut root: &mut tile::Node<windef::HWND>,
    window: windef::HWND,
    prev_window: windef::HWND,
) {
    let focused_node = tile::find_node(&mut root, prev_window);
    if let Some(last_focused) = focused_node {
        tile::tile(last_focused, tile::Orientation::Vertical, window);
    } else {
        tile::tile(&mut root, tile::Orientation::Vertical, window);
    }
    redraw_nodes(&root);
}

fn untile_window(mut root: &mut tile::Node<windef::HWND>, window: windef::HWND) {
    tile::untile(&mut root, &window);
    redraw_nodes(&root);
}

fn tile_existing_windows(
    mut windows: Vec<windef::HWND>,
    dim: tile::Dimensions,
) -> tile::Node<windef::HWND> {
    let mut root: tile::Node<windef::HWND> = tile::Node {
        node_type: tile::NodeType::Empty,
        dim: dim,
    };

    while !windows.is_empty() {
        tile::tile(&mut root, tile::Orientation::Horizontal, windows.remove(0));
    }

    root
}

fn redraw_nodes(root: &tile::Node<windef::HWND>) {
    match &root.node_type {
        tile::NodeType::Separator(_, left_child, right_child) => {
            redraw_nodes(left_child);
            redraw_nodes(right_child);
        }
        tile::NodeType::Window(hwnd) => {
            internal::show_window(hwnd.clone());
            internal::set_window_pos(
                hwnd.clone(),
                root.dim.x.0,
                root.dim.y.0,
                root.dim.x.1,
                root.dim.y.1,
            );
        }
        tile::NodeType::Empty => return,
    }
}
