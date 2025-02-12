// src/config_manager.rs

use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::{channel, TryRecvError};
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::watch;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemotePylonConfig {
    pub ip: String,
    pub port: u16,
    pub token: String,
    // New optional name for a remote pylon.
    pub name: Option<String>,
    // New: location string (e.g. "Tampa, FL")
    pub location: Option<String>,
    // New: description string
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // The default local port Pylon will try to use.
    pub local_port: Option<u16>,
    // The token that remote clients must supply when polling /api/metrics.
    pub token: String,
    // New: the name of this local Pylon.
    pub name: Option<String>,
    // A list of remote Pylon instances to poll.
    pub remote_pylons: Option<Vec<RemotePylonConfig>>,
    // New: local description (optional)
    pub description: Option<String>,
    // New: local location (optional)
    pub location: Option<String>,

    // NEW: auto-update configuration options
    pub auto_update: Option<bool>,
    pub master_update_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_port: Some(6989),
            token: "default_token".into(),
            name: Some("Local Pylon".into()),
            remote_pylons: None,
            description: None,
            location: None,
            auto_update: Some(false), // disabled by default
            master_update_url: Some("https://brinstar.top/pylon".into()),
        }
    }
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;
    settings.try_deserialize::<Config>()
}

/// Now accepts a shutdown receiver so it can exit gracefully.
pub async fn watch_config(config_arc: Arc<RwLock<Config>>, shutdown: watch::Receiver<bool>) {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    if let Err(e) = watcher.watch("config.toml", RecursiveMode::NonRecursive) {
        println!("Failed to watch config.toml: {}", e);
        return;
    }

    loop {
        if *shutdown.borrow() {
            println!("Shutting down config watcher.");
            break;
        }

        match rx.try_recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(path) => {
                    println!("Config file changed: {:?}", path);
                    if let Ok(new_config) = load_config() {
                        let mut config_lock = config_arc.write().unwrap();
                        *config_lock = new_config;
                        println!("Config reloaded.");
                    } else {
                        println!("Failed to reload config.");
                    }
                },
                _ => {},
            },
            Err(TryRecvError::Empty) => {
                sleep(Duration::from_secs(1)).await;
            },
            Err(TryRecvError::Disconnected) => {
                println!("Config watcher disconnected.");
                break;
            }
        }
    }
}

pub fn save_config(config: &Config) -> Result<(), std::io::Error> {
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write("config.toml", toml_str)
}
