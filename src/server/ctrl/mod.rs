mod rpc_methods;

use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::json;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::qcow2::Qcow2;

pub fn start_ctrl_server(qcow: Arc<Mutex<Qcow2>>) {
    info!("Starting controller on localhost:1234");
    info!("  > ctrl-c to quit, ");
    let help =
        r#"echo -n '{ "jsonrpc": "2.0", "method": "discover", "id": 1 }' | nc localhost 1234"#;
    info!("  > {}", help);

    let listener =
        TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|_| panic!("failed to bind listener"));

    for stream in listener.incoming() {
        let qcow_clone = Arc::clone(&qcow);
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_connection(stream, qcow_clone);
                });
            }
            Err(e) => error!("failed to get incoming connection: {}", e),
        }
    }
}

// https://www.jsonrpc.org/specification
// Currently we don't need parameters
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    id: u64,
    #[serde(default)]
    params: serde_json::Value,
}

fn handle_connection(mut stream: TcpStream, qcow: Arc<Mutex<Qcow2>>) {
    let rpc_methods = rpc_methods::init_once();

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

    let response = if let Some(handler) = rpc_methods.get(request.method.as_str()) {
        json!({
            "jsonrpc": "2.0",
            "result": handler(&qcow, &request.params),
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
