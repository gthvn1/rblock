use log::{debug, error, info};
use std::io::Write;
use std::net::{TcpListener, TcpStream};

pub fn start_nbd_server() {
    info!("Starting nbd server on localhost:10809");
    info!("  > ctrl-c to quit, ");

    let listener =
        TcpListener::bind("127.0.0.1:10809").unwrap_or_else(|_| panic!("failed to bind listener"));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => error!("failed to get incoming connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    const NBD_MAGIC: u64 = 0x4e42444d41474943;
    const IHAVEOPT: u64 = 0x49484156454f5054;
    const FLAGS: u16 = 1; // for example: export name required

    debug!("Sending new handshake");
    let mut handshake = Vec::new();
    handshake.extend_from_slice(&NBD_MAGIC.to_be_bytes());
    handshake.extend_from_slice(&IHAVEOPT.to_be_bytes());
    handshake.extend_from_slice(&FLAGS.to_be_bytes());
    stream.write_all(&handshake).unwrap();
}
