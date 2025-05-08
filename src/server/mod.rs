pub mod ctrl;
pub mod nbd;

use ctrl::start_ctrl_server;
use nbd::start_nbd_server;

pub fn start_servers() {
    std::thread::spawn(move || {
        start_nbd_server();
    });

    std::thread::spawn(move || {
        start_ctrl_server();
    });

    // Prevent the main thread from exiting
    loop {
        std::thread::park();
    }
}
