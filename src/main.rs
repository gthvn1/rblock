use std::{fs, net::TcpListener};

use rblock::qcow2::Qcow2;

const IFNAME: &str = "samples/input.txt";
const OFNAME: &str = "samples/output.txt";
const QCOWFNAME: &str = "samples/disk.qcow2";

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

fn main() {
    println!("Testing QCOW2");
    let mut qcow = Qcow2::new(QCOWFNAME).expect("Failed to read qcow file");
    if qcow.is_qcow() {
        println!("QCOW has valid magic");
    } else {
        println!("QCOW has not a valid magic");
    }

    // Todo: create a server with an endpoint where we can post a request and it
    //       returns the content transformed using rot13
    let listener =
        TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|_| panic!("failed to bind listener"));

    let buf = fs::read(IFNAME).unwrap_or_else(|_| panic!("Failed to read {}", IFNAME));
    fs::write(OFNAME, rot13(buf)).unwrap_or_else(|_| panic!("Failed to write {}", OFNAME));

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => println!("incoming connection"),
            Err(_) => println!("failed to get incoming connection"),
        }
    }
}
