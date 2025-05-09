use rblock::server::start_servers;
use std::env;

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

    start_servers(&fname);
}
