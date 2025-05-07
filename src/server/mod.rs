use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::json;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

// https://www.jsonrpc.org/specification
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    id: u64,
}

pub fn start_server() {
    info!("Starting server on localhost:1234");
    info!("  > ctrl-c to quit, ");
    // echo -n '{ "jsonrpc": "2.0", "method": "ping", "id": 1 }' | nc localhost 1234
    let help = r#"echo -n '{ "jsonrpc": "2.0", "method": "ping", "id": 1 }' | nc localhost 1234"#;
    info!("  > {}", help);
    let listener =
        TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|_| panic!("failed to bind listener"));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => error!("failed to get incoming connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let parse_error = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32700,
            "message": "Parse error"
        },
        "id": null
    });
    let mut buf: [u8; 1024] = [0; 1024];
    let sz = match stream.read(&mut buf) {
        Ok(0) | Err(_) => {
            let _ = stream.write_all(parse_error.to_string().as_bytes());
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
        Ok(sz) => sz,
    };

    if sz == 1024 {
        warn!("Warning: buffer is full");
    }

    let invalid_request = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32600,
            "message": "Invalid Request"
        },
        "id": null
    });

    let request = match std::str::from_utf8(&buf[0..sz]) {
        Ok(r) => r,
        Err(_) => {
            let _ = stream.write_all(invalid_request.to_string().as_bytes());
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
    };

    debug!("request: {}", request);

    let request = match serde_json::from_str::<JsonRpcRequest>(request) {
        Ok(r) => r,
        Err(_) => {
            let _ = stream.write_all(invalid_request.to_string().as_bytes());
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }
    };

    // TODO: check JSON RPC version
    let _ = request.jsonrpc;

    // We just support "ping" command...
    let response = if request.method == "ping" {
        json!({
            "jsonrpc": "2.0",
            "result": "pong",
            "id":request.id,
        })
    } else {
        json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32601,
                "message": "Method not found"
            },
            "id": null
        })
    };

    let serialized = response.to_string();

    stream
        .write_all(serialized.as_bytes())
        .expect("Failed to write response to stream");
}
