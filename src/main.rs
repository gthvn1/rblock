use std::fs;

const IFNAME: &str = "samples/input.txt";
const OFNAME: &str = "samples/output.txt";

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
    let buf = fs::read(IFNAME).unwrap_or_else(|_| panic!("Failed to read {}", IFNAME));
    fs::write(OFNAME, rot13(buf)).unwrap_or_else(|_| panic!("Failed to write {}", OFNAME));
}
