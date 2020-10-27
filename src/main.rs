pub mod core;
pub mod tile;
pub mod internal;

fn main() {
    run();
}

#[cfg(windows)]
fn run() {
    let res = core::run();

    match res {
        Ok(_) => println!("process exited successfully"),
        Err(e) => println!("process exited with error: {}", e)
    }
}

#[cfg(not(windows))]
fn run() {
    println!("Please run this application on a desktop with Windows 10")
}
