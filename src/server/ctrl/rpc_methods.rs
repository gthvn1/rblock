use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::qcow2::Qcow2;

type RpcHandler = fn(&Arc<Mutex<Qcow2>>) -> serde_json::Value;
static RPC_METHODS: OnceLock<HashMap<&'static str, RpcHandler>> = OnceLock::new();

fn rpc_ping(_qcow: &Arc<Mutex<Qcow2>>) -> serde_json::Value {
    json!("pong")
}

fn rpc_get_backing_file(qcow: &Arc<Mutex<Qcow2>>) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    match q.backing_file() {
        None => json!("".to_string()),
        Some(s) => json!(s),
    }
}

fn rpc_version(qcow: &Arc<Mutex<Qcow2>>) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    json!(q.version())
}

// Method to list all available methods (RPC discover)
#[derive(Debug, serde::Serialize)]
struct RpcMethodInfo {
    name: &'static str,
    parameters: Vec<String>, // You could use a more complex type depending on how you represent parameters
    return_type: String, // Return type as a string for simplicity (e.g., "string", "integer", etc.)
}

fn rpc_discover(_qcow: &Arc<Mutex<Qcow2>>) -> serde_json::Value {
    let methods = init_once();
    let method_infos: Vec<RpcMethodInfo> = methods
        .keys()
        .map(|&method_name| {
            match method_name {
                "ping" => RpcMethodInfo {
                    name: method_name,
                    parameters: vec![], // No parameters
                    return_type: "string".to_string(),
                },
                "get_backing_file" => RpcMethodInfo {
                    name: method_name,
                    parameters: vec![], // No parameters
                    return_type: "string".to_string(),
                },
                "version" => RpcMethodInfo {
                    name: method_name,
                    parameters: vec![], // No parameters
                    return_type: "int".to_string(),
                },

                "discover" => RpcMethodInfo {
                    name: method_name,
                    parameters: vec![], // No parameters
                    return_type: "object".to_string(),
                },
                _ => RpcMethodInfo {
                    name: method_name,
                    parameters: vec![], // Default to no parameters for unknown methods
                    return_type: "unknown".to_string(),
                },
            }
        })
        .collect();

    json!(method_infos)
}

pub fn init_once() -> &'static HashMap<&'static str, RpcHandler> {
    RPC_METHODS.get_or_init(|| {
        let mut map: HashMap<&'static str, RpcHandler> = HashMap::new();
        map.insert("ping", rpc_ping as RpcHandler);
        map.insert("get_backing_file", rpc_get_backing_file as RpcHandler);
        map.insert("version", rpc_version as RpcHandler);
        map.insert("discover", rpc_discover as RpcHandler);
        map
    })
}
