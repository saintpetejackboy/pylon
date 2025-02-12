use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};

use tokio::sync::watch;
use tokio::time::{sleep, Duration};

use reqwest::Client;
use crate::config_manager::Config;

/// Disables auto-update by setting the auto_update flag to false.
fn disable_auto_update(config_arc: &Arc<RwLock<Config>>) {
    if let Ok(mut cfg) = config_arc.write() {
        cfg.auto_update = Some(false);
    }
    println!("*** Auto update disabled due to update error. ***");
}

/// Checks for a new version and, if found, downloads and installs it.
/// This updater backs up the current binary and then replaces it.
/// Finally, it uses `sudo systemctl restart pylon` to restart the service.
pub async fn check_for_update(config_arc: Arc<RwLock<Config>>) -> Result<bool, Box<dyn std::error::Error>> {
    println!("================== Starting Update Check ==================");
    let client = Client::new();

    // Read configuration.
    let config = config_arc.read().unwrap().clone();
    if !config.auto_update.unwrap_or(false) {
        println!("Auto update is disabled in configuration.");
        return Ok(false);
    }

    let master_update_url = config.master_update_url.unwrap_or_default();
    if master_update_url.is_empty() {
        println!("Master update URL not configured. Exiting update process.");
        return Ok(false);
    }
    println!("Master update URL: {}", master_update_url);

    // Prepare update URLs.
    let version_url = format!("{}?action=version", master_update_url);
    let binary_url  = format!("{}?action=binary", master_update_url);

    println!("Checking for updates from: {}", version_url);

    // Fetch version info.
    let resp = match client.get(&version_url).send().await {
        Ok(r) => r,
        Err(e) => {
            println!("Error fetching version info: {}", e);
            return Ok(false);
        }
    };

    if !resp.status().is_success() {
        println!("Failed to fetch version info: HTTP {}", resp.status());
        return Ok(false);
    }

    let latest_version = resp.text().await?.trim().to_string();
    println!("Latest version from server: {}", latest_version);

    if latest_version == crate::server::PYLON_VERSION {
        println!("No update needed. Current version ({}) is up-to-date.", crate::server::PYLON_VERSION);
        return Ok(false);
    }
    println!("Update available: current version {} vs. new version {}", crate::server::PYLON_VERSION, latest_version);

    // Request the new binary.
    println!("Requesting new binary from: {}", binary_url);
    let bin_resp = match client.get(&binary_url).send().await {
        Ok(r) => r,
        Err(e) => {
            println!("Error requesting new binary: {}", e);
            return Ok(false);
        }
    };

    if !bin_resp.status().is_success() {
        println!("Failed to download new binary: HTTP {}", bin_resp.status());
        return Ok(false);
    }

    let data = match bin_resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            println!("Error reading binary data: {}", e);
            return Ok(false);
        }
    };

    println!("Downloaded binary data: {} bytes", data.len());
    if data.len() == 0 {
        println!("Error: Downloaded binary is empty!");
        return Ok(false);
    }

    // Write the new binary to a temporary file.
    let mut tmp_path: PathBuf = env::temp_dir();
    tmp_path.push("pylon_new");
    println!("Writing new binary to temporary file: {:?}", tmp_path);

    if let Err(e) = fs::write(&tmp_path, &data) {
        println!("Failed to write new binary to {:?}: {}", tmp_path, e);
        if e.kind() == io::ErrorKind::PermissionDenied {
            disable_auto_update(&config_arc);
        }
        return Ok(false);
    }

    // Set executable permissions on the new binary.
    println!("Setting executable permissions on temporary binary: {:?}", tmp_path);
    if let Err(e) = Command::new("chmod").arg("+x").arg(&tmp_path).status() {
        println!("Failed to set executable permissions on {:?}: {}", tmp_path, e);
        disable_auto_update(&config_arc);
        return Ok(false);
    }

    // Check that systemctl is available.
    println!("Verifying systemctl availability...");
    if Command::new("systemctl").arg("--version").output().is_err() {
        println!("systemctl not available. Disabling auto-update.");
        disable_auto_update(&config_arc);
        return Ok(false);
    }

    // Determine current executable path.
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            println!("Could not determine current executable path: {}", e);
            disable_auto_update(&config_arc);
            return Ok(false);
        }
    };
    println!("Current binary path: {:?}", current_exe);

    // Verify that the parent directory is writable.
    if let Some(parent) = current_exe.parent() {
        println!("Checking write permissions for directory: {:?}", parent);
        let test_path = parent.join("temp_test_file");
        match fs::write(&test_path, b"test") {
            Ok(_) => {
                println!("Directory is writable. Removing test file.");
                let _ = fs::remove_file(&test_path);
            },
            Err(e) => {
                println!("Directory {:?} is not writable: {}", parent, e);
                disable_auto_update(&config_arc);
                return Ok(false);
            }
        }
    } else {
        println!("Unable to determine parent directory for binary.");
        disable_auto_update(&config_arc);
        return Ok(false);
    }

    // Backup current binary.
    let backup_exe = current_exe.with_extension("old");
    println!("Backing up current binary: {:?} -> {:?}", current_exe, backup_exe);
    if let Err(e) = fs::rename(&current_exe, &backup_exe) {
        println!("Failed to backup current binary: {}", e);
        disable_auto_update(&config_arc);
        return Ok(false);
    }

    // Replace current binary with the new one.
    println!("Replacing current binary with new binary.");
    if let Err(e) = fs::rename(&tmp_path, &current_exe) {
        println!("Failed to move new binary into place: {}", e);
        // Attempt to restore backup.
        let _ = fs::rename(&backup_exe, &current_exe);
        disable_auto_update(&config_arc);
        return Ok(false);
    }
    println!("New binary successfully installed at {:?}", current_exe);

    // Restart the service using sudo systemctl.
    println!("Attempting to restart the service via sudo systemctl restart pylon");
    let restart_status = Command::new("sudo")
        .arg("systemctl")
        .arg("restart")
        .arg("pylon")
        .status();

    match restart_status {
        Ok(status) => {
            // Treat a SIGTERM exit (signal 15) as a successful restart because
            // systemd sends SIGTERM to stop the old instance.
            if status.success() || was_sigterm(&status) {
                println!("Service restart triggered successfully.");
                println!("================== Update Process Completed ==================");
                Ok(true)
            } else {
                println!("systemctl restart returned non-zero exit code: {}", status);
                disable_auto_update(&config_arc);
                println!("================== Update Process Failed ==================");
                Ok(false)
            }
        },
        Err(e) => {
            println!("Failed to execute sudo systemctl restart: {}", e);
            disable_auto_update(&config_arc);
            println!("================== Update Process Failed ==================");
            Ok(false)
        }
    }
}

#[cfg(unix)]
fn was_sigterm(status: &std::process::ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt;
    status.signal() == Some(15)
}

#[cfg(not(unix))]
fn was_sigterm(_status: &std::process::ExitStatus) -> bool {
    false
}

/// Runs the auto-update loop:
/// - Checks for an update immediately, then every 24 hours.
/// - Exits if auto-update is disabled or a shutdown signal is received.
pub async fn auto_update_loop(config_arc: Arc<RwLock<Config>>, mut shutdown: watch::Receiver<bool>) {
    println!("================== Starting Auto Update Loop ==================");
    let _ = check_for_update(Arc::clone(&config_arc)).await;

    loop {
        if !config_arc.read().unwrap().auto_update.unwrap_or(false) {
            println!("Auto update disabled. Exiting update loop.");
            break;
        }
        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    println!("Shutdown signal received. Exiting auto update loop.");
                    break;
                }
            },
            _ = sleep(Duration::from_secs(24 * 3600)) => {
                println!("24 hours elapsed. Checking for updates.");
                let _ = check_for_update(Arc::clone(&config_arc)).await;
            }
        }
    }
    println!("================== Auto Update Loop Terminated ==================");
}
