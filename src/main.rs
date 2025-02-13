// main.rs

mod config_manager;
mod system_info;
mod remote;
mod server;
mod updater; // <-- New updater module

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use config_manager::Config;
use system_info::SystemData;
use remote::RemoteStatus;
use server::AppState;
use tokio::sync::watch;

fn ensure_config_exists() -> std::io::Result<()> {
    let config_path = "config.toml";
    if !Path::new(config_path).exists() {
        const DEFAULT_CONFIG: &str = include_str!("../config_default.toml");
        fs::write(config_path, DEFAULT_CONFIG)?;
        println!("Default configuration written to '{}'.", config_path);
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = ensure_config_exists() {
        eprintln!("Failed to ensure config file exists: {}", e);
    }

    let initial_config = match config_manager::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Failed to load config: {}. Using default.", e);
            Config::default()
        }
    };
    let config = Arc::new(RwLock::new(initial_config));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let config_clone = Arc::clone(&config);
    tokio::spawn(config_manager::watch_config(config_clone, shutdown_rx.clone()));

    let system_data = Arc::new(Mutex::new(SystemData::new()));
    let system_data_clone = Arc::clone(&system_data);
    // --- Pass the config into the system metrics poller ---
    tokio::spawn(system_info::poll_system_metrics(system_data_clone, Arc::clone(&config), shutdown_rx.clone()));

    let remote_statuses = Arc::new(Mutex::new(HashMap::<String, RemoteStatus>::new()));
    let config_clone2 = Arc::clone(&config);
    let remote_statuses_clone = Arc::clone(&remote_statuses);
    tokio::spawn(remote::poll_remote_pylons(config_clone2, remote_statuses_clone, shutdown_rx.clone()));

    let config_clone3 = Arc::clone(&config);
    tokio::spawn(updater::auto_update_loop(config_clone3, shutdown_rx.clone()));

    let base_port = config.read().unwrap().local_port.unwrap_or(6989);
    let server_port = server::find_open_port(base_port).await;
    if server_port != base_port {
        println!("Port {} was in use. Running on port {} instead.", base_port, server_port);
    }

    let state = AppState {
        config: Arc::clone(&config),
        system_data: Arc::clone(&system_data),
        remote_statuses: Arc::clone(&remote_statuses),
    };

    let server = server::run_server(server_port, state);

    tokio::select! {
        res = server => {
            res
        },
        _ = tokio::signal::ctrl_c() => {
            println!("Received ctrl+c, shutting down gracefully...");
            let _ = shutdown_tx.send(true);
            Ok(())
        }
    }
}
