use std::fs;

const IFNAME: &str = "samples/input.txt";
const OFNAME: &str = "samples/output.txt";

fn main() {
    let buf = fs::read(IFNAME).unwrap_or_else(|_| panic!("Failed to read {}", IFNAME));
    fs::write(OFNAME, buf).unwrap_or_else(|_| panic!("Failed to read {}", OFNAME));
}
