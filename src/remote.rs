// src/remote.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::watch;
use crate::config_manager::Config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteStatus {
    pub ip: String,
    pub port: u16,
    pub last_seen: Option<DateTime<Utc>>,
    pub data: Option<Value>, // remote metrics as JSON
    pub online: bool,
    // New: include the name from the configuration (if any)
    pub name: Option<String>,
}

/// Now accepts a shutdown receiver so it can exit gracefully.
/// Also performs peer discovery by reading the "remote_pylons" field returned
/// by remote servers and adding new ones to the polling list.
pub async fn poll_remote_pylons(
    config_arc: Arc<std::sync::RwLock<Config>>,
    remote_statuses: Arc<Mutex<HashMap<String, RemoteStatus>>>,
    mut shutdown: watch::Receiver<bool>
) {
    let client = Client::new();
    // Discovered peers (not in the initial config)
    let mut discovered_peers: Vec<crate::config_manager::RemotePylonConfig> = Vec::new();
    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    println!("Shutting down remote poller.");
                    break;
                }
            },
            _ = async {
                // Build list of all remotes to poll: config.remote_pylons + discovered_peers (deduplicated)
                let mut all_remotes: Vec<crate::config_manager::RemotePylonConfig> = Vec::new();
                {
                    let config = config_arc.read().unwrap().clone();
                    if let Some(remote_list) = config.remote_pylons {
                        all_remotes.extend(remote_list);
                    }
                }
                // Add discovered peers that are not already in all_remotes.
                for peer in discovered_peers.iter() {
                    let key = format!("{}:{}", peer.ip, peer.port);
                    if !all_remotes.iter().any(|r| format!("{}:{}", r.ip, r.port) == key) {
                        all_remotes.push(peer.clone());
                    }
                }
                
                for remote in all_remotes {
                    let key = format!("{}:{}", remote.ip, remote.port);
                    let url = format!("http://{}:{}/api/metrics", remote.ip, remote.port);
                    let req = client.get(&url)
                        .bearer_auth(remote.token.clone())
                        .timeout(Duration::from_secs(5))
                        .send()
                        .await;
                    let mut status = RemoteStatus {
                        ip: remote.ip.clone(),
                        port: remote.port,
                        last_seen: None,
                        data: None,
                        online: false,
                        name: remote.name.clone(),
                    };
                    match req {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                if let Ok(json_data) = resp.json::<Value>().await {
                                    status.data = Some(json_data.clone());
                                    status.online = true;
                                    status.last_seen = Some(Utc::now());
                                    // Peer discovery: if the remote data contains "remote_pylons", add them
                                    if let Some(peers) = json_data.get("remote_pylons") {
                                        if let Some(array) = peers.as_array() {
                                            for peer_val in array {
                                                if let Ok(peer_config) = serde_json::from_value::<crate::config_manager::RemotePylonConfig>(peer_val.clone()) {
                                                    let peer_key = format!("{}:{}", peer_config.ip, peer_config.port);
                                                    if !discovered_peers.iter().any(|p| format!("{}:{}", p.ip, p.port) == peer_key) &&
                                                       peer_key != key { // avoid self
                                                        discovered_peers.push(peer_config);
                                                        println!("Discovered new peer: {}", peer_key);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            println!("Error connecting to remote {}: {}", key, e);
                        }
                    }
                    let mut statuses = remote_statuses.lock().unwrap();
                    statuses.insert(key, status);
                }
            } => {}
        }
        sleep(Duration::from_secs(10)).await;
    }
}
