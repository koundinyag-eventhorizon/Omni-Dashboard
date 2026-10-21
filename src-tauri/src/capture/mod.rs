pub mod accessibility;
pub mod audio;
pub mod screen;

pub fn init_capture_system() {
    println!("🚀 Capture engine online. Spawning OS thread for active window tracking...");

    // Spawn a dedicated thread for the accessibility capture loop
    std::thread::spawn(|| {
        accessibility::start_loop();
    });
}
