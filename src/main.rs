use log::{debug, error};
use rblock::qcow2::Qcow2;
use rblock::server::start_server;
use std::{env, fs};

fn rot13(v: Vec<u8>) -> Vec<u8> {
    let mut rotv: Vec<u8> = Vec::new();
    for c in v.into_iter() {
        let r = match c {
            b'A'..=b'Z' => b'A' + ((c - b'A' + 13) % 26),
            b'a'..=b'z' => b'a' + ((c - b'a' + 13) % 26),
            _ => c,
        };

        rotv.push(r)
    }

    rotv
}

fn testing_qcow2(fname: &str) {
    let qcow = Qcow2::new(fname).expect("Failed to read qcow file");
    debug!("Detected qcow version : {}", qcow.version());
    debug!("Backing file          : {:?}", qcow.backing_file());
}

fn testing_rot13() {
    const IFNAME: &str = "samples/input.txt";
    const OFNAME: &str = "samples/output.txt";

    debug!("Testing ROT13: {} -> {}", IFNAME, OFNAME);
    let buf = match fs::read(IFNAME) {
        Ok(v) => v,
        Err(e) => {
            error!("failed to read {}: {}", IFNAME, e);
            return;
        }
    };

    fs::write(OFNAME, rot13(buf)).unwrap_or_else(|e| {
        error!("Failed to write {}: {}", OFNAME, e);
    })
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .init();

    const QCOWFNAME: &str = "samples/disk.qcow2";
    let mut arguments = env::args();

    // Skip the first argument that is the name of program
    let _progname = arguments.next();

    let fname = match arguments.next() {
        None => QCOWFNAME.to_string(),
        Some(f) => f,
    };

    debug!("qcow file: {}", fname);
    testing_qcow2(&fname);
    testing_rot13();
    start_server();
}
