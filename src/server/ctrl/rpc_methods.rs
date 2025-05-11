use base64::{Engine as _, engine::general_purpose};
use log::warn;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::qcow2::Qcow2;

type RpcHandler = fn(&Arc<Mutex<Qcow2>>, &serde_json::Value) -> serde_json::Value;
static RPC_METHODS: OnceLock<HashMap<&'static str, RpcHandler>> = OnceLock::new();

fn rpc_cluster_size(qcow: &Arc<Mutex<Qcow2>>, _params: &serde_json::Value) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    json!(q.cluster_size())
}

fn rpc_get_backing_file(
    qcow: &Arc<Mutex<Qcow2>>,
    _params: &serde_json::Value,
) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    match q.backing_file() {
        None => json!("".to_string()),
        Some(s) => json!(s),
    }
}

fn rpc_l1_size(qcow: &Arc<Mutex<Qcow2>>, _params: &serde_json::Value) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    json!(q.l1_size())
}

fn rpc_l1_table_offset(qcow: &Arc<Mutex<Qcow2>>, _params: &serde_json::Value) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    json!(q.l1_table_offset())
}

fn rpc_ping(_qcow: &Arc<Mutex<Qcow2>>, _params: &serde_json::Value) -> serde_json::Value {
    json!("pong")
}

fn rpc_version(qcow: &Arc<Mutex<Qcow2>>, _params: &serde_json::Value) -> serde_json::Value {
    let q = qcow.lock().unwrap();
    json!(q.version())
}

fn rpc_read_guest_cluster(
    qcow: &Arc<Mutex<Qcow2>>,
    params: &serde_json::Value,
) -> serde_json::Value {
    let cluster_index = match params.get("cluster") {
        Some(v) => v.as_u64().unwrap_or_else(|| {
            // let's default to 0 for now
            warn!("Failed to get cluster index, default to 0");
            0
        }),
        None => {
            warn!("No cluster index passed as parameter, default to 0");
            0
        }
    };

    let q = qcow.lock().unwrap();
    let data = q.read_guest_cluster(cluster_index);
    let encoded = general_purpose::STANDARD.encode(data);
    json!(encoded)
}

// Method to list all available methods (RPC discover)
#[derive(Debug, serde::Serialize)]
struct RpcMethodInfo {
    name: &'static str,
    description: &'static str,
    params : Vec<(&'static str, &'static str)>,
    return_type: &'static str, // Return type as a string for simplicity (e.g., "string", "integer", etc.)
}

fn rpc_discover(_qcow: &Arc<Mutex<Qcow2>>, _params: &serde_json::Value) -> serde_json::Value {
    let methods = init_once();
    let method_infos: Vec<RpcMethodInfo> = methods
        .keys()
        .map(|&method_name| match method_name {
            "cluster_size" => RpcMethodInfo {
                name: method_name,
                description: "Cluster size",
                params: vec![],
                return_type: "integer",
            },
            "discover" => RpcMethodInfo {
                name: method_name,
                description: "List all available methods",
                params: vec![],
                return_type: "array of methods info objects",
            },
            "get_backing_file" => RpcMethodInfo {
                name: method_name,
                description: "Get backing file name",
                params: vec![],
                return_type: "string",
            },
            "l1_size" => RpcMethodInfo {
                name: method_name,
                description: "Number of entries in L1 table",
                params: vec![],
                return_type: "integer",
            },
            "l1_table_offset" => RpcMethodInfo {
                name: method_name,
                description: "Offset of L1 table",
                params: vec![],
                return_type: "integer (64-bit)",
            },
            "ping" => RpcMethodInfo {
                name: method_name,
                description: "Ping, check if server is running",
                params: vec![],
                return_type: "string",
            },
            "read_guest_cluster" => RpcMethodInfo {
                name: method_name,
                description: "Read guest cluster",
                params: vec![("cluster", "integer")],
                return_type: "Base64 encoded string",
            },
            "version" => RpcMethodInfo {
                name: method_name,
                description: "Version of the qcow2 file",
                params: vec![],
                return_type: "integer",
            },
            _ => RpcMethodInfo {
                name: method_name,
                description: "Unknown method",
                params: vec![],
                return_type: "unknown",
            },
        })
        .collect();

    json!(method_infos)
}

pub fn init_once() -> &'static HashMap<&'static str, RpcHandler> {
    RPC_METHODS.get_or_init(|| {
        let mut map: HashMap<&'static str, RpcHandler> = HashMap::new();
        map.insert("cluster_size", rpc_cluster_size as RpcHandler);
        map.insert("discover", rpc_discover as RpcHandler);
        map.insert("get_backing_file", rpc_get_backing_file as RpcHandler);
        map.insert("l1_size", rpc_l1_size as RpcHandler);
        map.insert("l1_table_offset", rpc_l1_table_offset as RpcHandler);
        map.insert("ping", rpc_ping as RpcHandler);
        map.insert("read_guest_cluster", rpc_read_guest_cluster as RpcHandler);
        map.insert("version", rpc_version as RpcHandler);
        map
    })
}
