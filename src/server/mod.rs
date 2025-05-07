use serde::Deserialize;

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
    println!("Starting server on localhost:1234");
    println!("  > ctrl-c to quit, ");
    // echo -n '{ "jsonrpc": "2.0", "method": "ping", "id": 1 }' | nc localhost 1234
    let help = r#"echo -n '{ "jsonrpc": "2.0", "method": "ping", "id": 1 }' | nc localhost 1234"#;
    println!("  > {}", help);
    let listener =
        TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|_| panic!("failed to bind listener"));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(_) => println!("failed to get incoming connection"),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];
    let sz = stream
        .read(&mut buf)
        .expect("Failed to read data from stream");

    if sz == 0 {
        println!("read empty stream... closing connection");
        return;
    }

    if sz == 1024 {
        println!("Warning: buffer is full");
    }

    let request = std::str::from_utf8(&buf[0..sz]).expect("Invalid UTF-8 request");
    println!("request: {}", request);

    let request =
        serde_json::from_str::<JsonRpcRequest>(request).expect("Failed to deserialize request");

    let _ = request.jsonrpc;

    let response = if request.method == "ping" {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": "pong",
            "id":request.id,
        })
    } else {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": "unknown method",
            "id":request.id,
        })
    };

    let serialized = response.to_string();

    stream
        .write(serialized.as_bytes())
        .expect("Failed to write response to stream");
}
