use log::{debug, error, info};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::qcow2::Qcow2;

enum NbdOpt {
    ExportName = 1,
    Abort,
    List,
    PeekExport,
    Starttls,
    Info,
    Go,
    StructuredReply,
    ListMetaContext,
    SetMetaContext,
    ExtendedHeaders,
}

impl TryFrom<u32> for NbdOpt {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NbdOpt::ExportName),
            2 => Ok(NbdOpt::Abort),
            3 => Ok(NbdOpt::List),
            4 => Ok(NbdOpt::PeekExport),
            5 => Ok(NbdOpt::Starttls),
            6 => Ok(NbdOpt::Info),
            7 => Ok(NbdOpt::Go),
            8 => Ok(NbdOpt::StructuredReply),
            9 => Ok(NbdOpt::ListMetaContext),
            10 => Ok(NbdOpt::SetMetaContext),
            11 => Ok(NbdOpt::ExtendedHeaders),
            _ => Err(()),
        }
    }
}

// enum OptionReplyTypes {
//     NbdRepAck = 1,
//     NbdRepServer,
//     NbdRepInfo,
//     NbdRepMetaContext,
// }

pub fn start_nbd_server(_qcow: Arc<Mutex<Qcow2>>) {
    info!("Starting nbd server on localhost:10809");
    info!("  > ctrl-c to quit, ");

    let listener =
        TcpListener::bind("127.0.0.1:10809").unwrap_or_else(|_| panic!("failed to bind listener"));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    // TODO: clone qcow to pass it to the handler
                    handle_connection(stream);
                });
            }
            Err(e) => error!("failed to get incoming connection: {}", e),
        }
    }
}

macro_rules! try_or_shutdown {
    ($stream:expr, $expr:expr, $msg:expr) => {
        if let Err(e) = $expr {
            error!("{}: {}", $msg, e);
            let _ = $stream.shutdown(std::net::Shutdown::Both);
            return;
        }
    };
}

// https://github.com/NetworkBlockDevice/nbd/blob/master/doc/proto.md
fn handle_connection(mut stream: TcpStream) {
    // 1. Send the handshake
    debug!("handshake begin");
    // Use newstyle negotiation:
    const NBD_MAGIC: u64 = 0x4e42444d41474943;
    const IHAVEOPT: u64 = 0x49484156454f5054;
    const NBD_FLAG_FIXED_NEWSTYLE: u16 = 0x1; // S: Handshake flag
    const NBD_FLAG_C_FIXED_NEWSTYLE: u32 = 0x1; // C: Handshake flag

    let mut handshake = Vec::new();
    handshake.extend_from_slice(&NBD_MAGIC.to_be_bytes());
    handshake.extend_from_slice(&IHAVEOPT.to_be_bytes());
    handshake.extend_from_slice(&NBD_FLAG_FIXED_NEWSTYLE.to_be_bytes());

    try_or_shutdown!(
        stream,
        stream.write_all(&handshake),
        "failed to send handshake"
    );

    debug!("handshake sent -> {:02x?}", handshake);

    try_or_shutdown!(stream, stream.flush(), "failed to flush stream");

    // 2. Read client flags (4 bytes)
    let mut buf: [u8; 4] = [0; 4];
    try_or_shutdown!(
        stream,
        stream.read_exact(&mut buf),
        "failed to read client flags"
    );

    let client_flags: u32 = u32::from_be_bytes(buf);
    debug!("read client flags: 0x{:08x}", client_flags);

    if client_flags & NBD_FLAG_C_FIXED_NEWSTYLE != NBD_FLAG_C_FIXED_NEWSTYLE {
        error!("client does not support fixed new style protocol: abort");
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return;
    }

    // This complete the initial phase of negotiation
    debug!("handshake end");

    // 3. Just wait (don't close) — the client will now send NBD_OPT_* commands
    // When NBD_OPT_EXPORT_NAME is negociated we can start the data exchange.
    // If We understand correctly the client can send one or more options.
    debug!("option haggling begin");

    // loop {
    let mut buf: [u8; 8] = [0; 8];
    try_or_shutdown!(
        stream,
        stream.read_exact(&mut buf),
        "failed to read IHAVEOPT"
    );

    let magic: u64 = u64::from_be_bytes(buf);
    if magic != IHAVEOPT {
        error!("expected IHAVEOPT but got {:16x}", magic);
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return;
    }

    // Now we are expecting:
    // C:32 bits, option
    // C:32 bits, length of optional data (unsigned)
    // C: any data needed for the chosen option of length
    let mut buf: [u8; 4] = [0; 4];
    try_or_shutdown!(stream, stream.read_exact(&mut buf), "failed to read option");
    let opt: u32 = u32::from_be_bytes(buf);

    try_or_shutdown!(stream, stream.read_exact(&mut buf), "failed to read length");
    let length: u32 = u32::from_be_bytes(buf);

    let mut data = vec![0u8; length as usize];
    try_or_shutdown!(stream, stream.read_exact(&mut data), "failed to read data");

    debug!("opt: 0x{:08x}, length: {}, data: {:x?}", opt, length, data);

    match NbdOpt::try_from(opt) {
        Ok(_) => debug!("TODO: respond to option..."),
        Err(_) => {
            error!("option is unknown");
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
    }
    // }

    debug!("option haggling end");

    debug!("transmission todo");
    let _ = stream.shutdown(std::net::Shutdown::Both);
}
