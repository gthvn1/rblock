pub mod ctrl;
pub mod nbd;

use ctrl::start_ctrl_server;
use nbd::start_nbd_server;

use crate::qcow2::Qcow2;

use log::debug;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_servers(fname: &str) {
    let qcow = Arc::new(Mutex::new(
        Qcow2::new(fname).expect("Failed to read qcow file"),
    ));

    {
        let q = qcow.lock().unwrap();
        debug!("Detected qcow version : {}", q.version());
        debug!("Backing file          : {:?}", q.backing_file());
    }

    let qcow_clone = Arc::clone(&qcow);
    thread::spawn(move || {
        start_nbd_server(qcow_clone);
    });

    let qcow_clone = Arc::clone(&qcow);
    thread::spawn(move || {
        start_ctrl_server(qcow_clone);
    });

    // Prevent the main thread from exiting
    loop {
        std::thread::park();
    }
}
