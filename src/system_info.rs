// src/system_info.rs

use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt, LoadAvg, ProcessExt, PidExt};
use tokio::time::{sleep, Duration};
use serde::Serialize;
use std::sync::{Mutex, Arc};

#[derive(Debug, Serialize, Clone)]
pub struct CachedInfo {
    pub os_version: String,
    pub apache_version: String,
    pub php_version: String,
    pub mariadb_version: String,
    pub rust_version: String,
    pub node_version: String,
    pub npm_version: String,
    pub processor: String,
    pub total_ram: u64,
    pub disk_capacity: u64,
    pub disk_usage: u64,
    pub boot_time: u64,
    // New: average and maximum CPU speed in MHz
    pub average_cpu_speed_mhz: u64,
    pub max_cpu_speed_mhz: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory: u64, // memory in KB
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct PolledMetrics {
    pub cpu_usage: f32,
    pub used_ram: u64,
    pub available_ram: u64,
    pub network_received: u64,
    pub network_transmitted: u64,
    pub uptime: u64,
    pub load_average: LoadAvg,
    pub swap_used: u64,
    pub swap_total: u64,
    pub disk_free: u64,
    pub disk_usage_percent: f32,
    pub services: Vec<ServiceStatus>,
    pub top_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone)]
pub struct SystemData {
    pub cached: CachedInfo,
    pub polled: PolledMetrics,
}

impl SystemData {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        // Refresh once to get initial values.
        sys.refresh_all();

        // Compute disk totals.
        let (total_capacity, total_usage) = sys.disks().iter().fold((0, 0), |(cap, used), disk| {
            (cap + disk.total_space(), used + (disk.total_space() - disk.available_space()))
        });

        // Compute CPU speed from all available CPUs.
        let cpus = sys.cpus();
        let (avg_cpu_speed, max_cpu_speed) = if !cpus.is_empty() {
            let total: u64 = cpus.iter().map(|cpu| cpu.frequency()).sum();
            let avg = total / (cpus.len() as u64);
            let max = cpus.iter().map(|cpu| cpu.frequency()).max().unwrap_or(avg);
            (avg, max)
        } else {
            (0, 0)
        };

        let cached = CachedInfo {
            os_version: get_os_version(),
            apache_version: get_command_version("apache2", "-v"),
            php_version: get_command_version("php", "-v"),
            mariadb_version: get_command_version("mysql", "--version"),
            rust_version: get_command_version("rustc", "--version"),
            node_version: get_command_version("node", "-v"),
            npm_version: get_command_version("npm", "-v"),
            processor: get_processor_info(),
            total_ram: sys.total_memory(),
            disk_capacity: total_capacity,
            disk_usage: total_usage,
            boot_time: sys.boot_time(),
            average_cpu_speed_mhz: avg_cpu_speed,
            max_cpu_speed_mhz: max_cpu_speed,
        };

        Self {
            cached,
            polled: PolledMetrics::default(),
        }
    }
}

fn get_os_version() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line.replace("PRETTY_NAME=", "").replace("\"", "");
            }
        }
    }
    "Unknown OS".to_string()
}

fn get_command_version(cmd: &str, arg: &str) -> String {
    if let Ok(output) = std::process::Command::new(cmd).arg(arg).output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).to_string();
            return s.lines().next().unwrap_or("Unknown").to_string();
        }
    }
    "Not installed".to_string()
}

fn get_processor_info() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return parts[1].trim().to_string();
                }
            }
        }
    }
    "Unknown Processor".to_string()
}

/// Poll system metrics (including the top five memory‐hungry processes)
pub async fn poll_system_metrics(data: Arc<Mutex<SystemData>>, mut shutdown: tokio::sync::watch::Receiver<bool>) {
    let mut sys = System::new_all();
    loop {
        if *shutdown.borrow() {
            println!("Shutting down system metrics poller.");
            break;
        }
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let used_ram = sys.used_memory();
        let available_ram = sys.available_memory();

        // Calculate disk usage.
        let (total_capacity, total_free) = sys.disks().iter().fold((0, 0), |(cap, free), disk| {
            (cap + disk.total_space(), free + disk.available_space())
        });
        let used_disk = if total_capacity > 0 { total_capacity - total_free } else { 0 };
        let disk_usage_percent = if total_capacity > 0 {
            used_disk as f32 / total_capacity as f32
        } else { 0.0 };

        // Sum network stats.
        let (mut network_received, mut network_transmitted) = (0, 0);
        for (_iface, data_net) in sys.networks() {
            network_received += data_net.received();
            network_transmitted += data_net.transmitted();
        }

        let uptime = sys.uptime();
        let load_average = sys.load_average();
        let swap_total = sys.total_swap();
        let swap_used = sys.used_swap();

        // Check for service statuses.
        let mut apache_running = false;
        let mut mariadb_running = false;
        for process in sys.processes().values() {
            let proc_name = process.name().to_lowercase();
            if proc_name.contains("apache2") {
                apache_running = true;
            }
            if proc_name.contains("mariadb") || proc_name.contains("mysql") {
                mariadb_running = true;
            }
        }
        let services = vec![
            ServiceStatus { name: "Apache2".into(), running: apache_running },
            ServiceStatus { name: "MariaDB".into(), running: mariadb_running },
        ];

        // Calculate top 5 processes by memory usage.
        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| b.memory().cmp(&a.memory()));
        let top_processes: Vec<ProcessInfo> = processes.iter().take(5).map(|process| ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_string(),
            memory: process.memory(),
        }).collect();

        // Update CPU speed info on each poll.
        let cpus = sys.cpus();
        let (avg_cpu_speed, max_cpu_speed) = if !cpus.is_empty() {
            let total: u64 = cpus.iter().map(|cpu| cpu.frequency()).sum();
            let avg = total / (cpus.len() as u64);
            let max = cpus.iter().map(|cpu| cpu.frequency()).max().unwrap_or(avg);
            (avg, max)
        } else {
            (0, 0)
        };

        {
            let mut data_lock = data.lock().unwrap();
            data_lock.polled.cpu_usage = cpu_usage;
            data_lock.polled.used_ram = used_ram;
            data_lock.polled.available_ram = available_ram;
            data_lock.polled.network_received = network_received;
            data_lock.polled.network_transmitted = network_transmitted;
            data_lock.polled.uptime = uptime;
            data_lock.polled.load_average = load_average;
            data_lock.polled.swap_total = swap_total;
            data_lock.polled.swap_used = swap_used;
            data_lock.polled.disk_free = total_free;
            data_lock.polled.disk_usage_percent = disk_usage_percent;
            data_lock.polled.services = services;
            data_lock.polled.top_processes = top_processes;
            data_lock.cached.average_cpu_speed_mhz = avg_cpu_speed;
            data_lock.cached.max_cpu_speed_mhz = max_cpu_speed;
        }

        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    println!("Shutting down system metrics poller.");
                    break;
                }
            },
            _ = sleep(Duration::from_secs(1)) => {}
        }
    }
}
