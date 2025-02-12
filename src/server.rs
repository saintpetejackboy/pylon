// src/server.rs

use actix_files::Files;
use actix_session::{Session, SessionMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;

use actix_web::{
    get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Logger,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::remote::RemoteStatus;
use crate::system_info::SystemData;

const PYLON_VERSION: &str = "0.2.1";

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<crate::config_manager::Config>>,
    pub system_data: Arc<Mutex<SystemData>>,
    pub remote_statuses: Arc<Mutex<HashMap<String, RemoteStatus>>>,
}

///
/// GET /
///
/// Serves the main HTML. Notice that we no longer expose the secret token.
///
#[get("/")]
async fn index() -> impl Responder {
    let html = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Pylon Dashboard 🚀✨</title>
  <link rel="stylesheet" href="/static/css/styles.css" />
  <link rel="icon" href="/static/favicon.ico" type="image/x-icon">
  <!-- Web Manifest -->
  <link rel="manifest" href="/static/site.webmanifest">
    <!-- Favicon -->
    <link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png">
    <link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png">
    <link rel="icon" type="image/png" sizes="192x192" href="/static/android-chrome-192x192.png">
    <link rel="icon" type="image/png" sizes="512x512" href="/static/android-chrome-512x512.png">
    <link rel="shortcut icon" href="/static/favicon.ico">
    <!-- Apple Touch Icon (for iOS devices) -->
    <link rel="apple-touch-icon" href="/static/apple-touch-icon.png">
    <!-- Theme Colors (for mobile browsers) -->
	
    <meta name="theme-color" content="black" /> 
    <meta name="msapplication-TileColor" content="black" />
  <script src="static/js/progressbar.min.js"></script>
  <script src="static/js/chart.js"></script>
</head>
<body>
  <div class="container">
    <!-- The pylon header (name, location, version) -->
    <h1 id="localPylonName">Pylon Dashboard</h1>
    
    <!-- Local Pylon Card -->
    <div class="card" id="localMetricsCard">
      <!-- Gauges for the local pylon -->
      <div class="gauges">
        <div class="gauge-container">
          <div id="cpuGauge" class="gauge"></div>
          <div class="gauge-label">⚡ CPU Usage</div>
        </div>
        <div class="gauge-container">
          <div id="ramGauge" class="gauge"></div>
          <div class="gauge-label">📊 RAM Usage</div>
          <div id="ramUsageText" style="margin-top: 8px; font-size: 1rem;"></div>
        </div>
        <div class="gauge-container">
          <div id="diskGauge" class="gauge"></div>
          <div class="gauge-label">💾 Disk Usage</div>
          <div id="diskUsageText" style="margin-top: 8px; font-size: 1rem;"></div>
        </div>
      </div>
      <!-- Local Services Status Lights -->
      <div class="services" id="servicesStatus">
        <!-- Filled dynamically -->
      </div>
    </div>
    
    <!-- Remote Pylons Header -->
    <h2>Remote Pylons 🌐</h2>
    <!-- Remote Pylons Container -->
    <div id="remoteContainer">
      <!-- Remote pylon cards appended dynamically -->
    </div>
  
    <!-- Network Throughput Card -->
    <div class="card" id="networkCard">
      <h2>Network Throughput 🌐</h2>
      <div class="network-chart-container">
        <canvas id="networkChart"></canvas>
      </div>
    </div>
	
    <!-- Admin Login Card -->
    <div class="card" id="adminLoginCard">
      <h2>Admin Access</h2>
      <input type="password" id="adminKeyInput" class="adminInput" placeholder="Enter Admin Key">
      <button id="adminKeySubmit" class="adminSubmit">Unlock Admin Features</button>
      <p id="adminError" style="color: red; display: none;">Incorrect key.</p>
    </div>
    
    <!-- Placeholder for Admin Content -->
    <div id="adminContent"></div>
  </div>
  
  <script type="module" src="/static/js/main.js"></script>
  
  <!-- Modal for Pylon Description -->
  <div id="pylonModal" class="modal">
    <div class="modal-content">
      <span class="close">&times;</span>
      <h2>Pylon Description</h2>
      <p id="modalDescription"></p>
    </div>
  </div>
</body>
</html>
"#;
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

///
/// POST /api/login
///
/// Accepts a JSON payload containing the admin token. If it matches the server–side token,
/// the session is marked as authenticated.
///
#[derive(Deserialize)]
struct LoginRequest {
    token: String,
}

#[post("/api/login")]
async fn login(
    data: web::Data<AppState>,
    login_req: web::Json<LoginRequest>,
    session: Session,
) -> impl Responder {
    let config = data.config.read().unwrap();
    if login_req.token == config.token {
        session.insert("admin_authenticated", true).unwrap();
        HttpResponse::Ok().json(json!({"status": "logged in"}))
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}))
    }
}

///
/// GET /api/admin-content
///
/// Returns admin–only HTML if the session has been authenticated.
///
#[get("/api/admin-content")]
async fn admin_content(session: Session) -> impl Responder {
    if let Ok(Some(true)) = session.get::<bool>("admin_authenticated") {
        let admin_html = r#"
      <!-- System Information Card -->
      <div class="card" id="systemInfoCard">
        <h2>System Information 📋</h2>
        <div id="systemDetails"></div>
      </div>
      
      <!-- Other Metrics Card -->
      <div class="card" id="otherMetricsCard">
        <h2>Other Metrics ⏱️</h2>
        <div>Uptime: <span id="uptime">0</span> seconds</div>
        <div>Load Average: <span id="loadAverage">0</span></div>
      </div>
      
      <!-- Top Processes Card -->
      <div class="card" id="topProcessesCard">
        <h2>Top 5 Processes by Memory</h2>
        <table id="topProcessesTable">
          <thead>
            <tr><th>PID</th><th>Name</th><th>Memory (MB)</th></tr>
          </thead>
          <tbody></tbody>
        </table>
      </div>
      
      <!-- Manage Remote Pylons Card -->
      <div class="card" id="managePylonsCard">
        <h2>Manage Remote Pylons</h2>
        <form id="pylonForm">
          <input type="text" id="pylonIp" placeholder="IP Address" required>
          <input type="number" id="pylonPort" placeholder="Port" required>
          <input type="text" id="pylonToken" placeholder="Token" required>
          <input type="text" id="pylonName" placeholder="Name (optional)">
          <button type="submit">Add Pylon</button>
        </form>
        <h3>Current Remote Pylons</h3>
        <ul id="pylonList"></ul>
      </div>
        "#;
        HttpResponse::Ok().content_type("text/html; charset=utf-8").body(admin_html)
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

///
/// GET /api/metrics
///
#[get("/api/metrics")]
async fn metrics(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let sys_data = data.system_data.lock().unwrap();
    let config = data.config.read().unwrap();
    let local_name = config.name.clone().unwrap_or_else(|| "Local Pylon".to_string());
    let local_description = config.description.clone().unwrap_or_else(|| "Sorry, no description was provided for this Pylon.".to_string());
    let local_location = config.location.clone().unwrap_or_else(|| "Unknown Location".to_string());
    let response = json!({
        "name": local_name,
        "description": local_description,
        "location": local_location,
        "version": PYLON_VERSION,
        "cached": sys_data.cached,
        "polled": sys_data.polled,
        "remote_pylons": config.remote_pylons,
    });
    HttpResponse::Ok().json(response)
}

///
/// GET /api/remotes
///
#[get("/api/remotes")]
async fn remotes(data: web::Data<AppState>) -> impl Responder {
    let statuses = data.remote_statuses.lock().unwrap();
    let response: Vec<_> = statuses.values().cloned().collect();
    HttpResponse::Ok().json(response)
}

///
/// GET /api/config/pylons
///
#[get("/api/config/pylons")]
async fn get_pylons(data: web::Data<AppState>) -> impl Responder {
    let config = data.config.read().unwrap();
    HttpResponse::Ok().json(&config.remote_pylons)
}

#[derive(Deserialize)]
struct RemovePylonRequest {
    ip: String,
    port: u16,
}

///
/// POST /api/config/pylons/add
///
#[post("/api/config/pylons/add")]
async fn add_pylon(
    data: web::Data<AppState>,
    new_pylon: web::Json<crate::config_manager::RemotePylonConfig>,
) -> impl Responder {
    let mut config = data.config.write().unwrap();
    if config.remote_pylons.is_none() {
        config.remote_pylons = Some(vec![]);
    }
    if let Some(ref mut pylons) = config.remote_pylons {
        pylons.push(new_pylon.into_inner());
    }
    match crate::config_manager::save_config(&config) {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "added"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

///
/// POST /api/config/pylons/remove
///
#[post("/api/config/pylons/remove")]
async fn remove_pylon(
    data: web::Data<AppState>,
    info: web::Json<RemovePylonRequest>,
) -> impl Responder {
    let mut config = data.config.write().unwrap();
    if let Some(ref mut pylons) = config.remote_pylons {
        pylons.retain(|p| !(p.ip == info.ip && p.port == info.port));
    }
    match crate::config_manager::save_config(&config) {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "removed"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

///
/// Helper: Check if a port is available
///
async fn port_available(port: u16) -> bool {
    use tokio::net::TcpListener;
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

///
/// Finds an open port starting with the given port.
///
pub async fn find_open_port(start_port: u16) -> u16 {
    let mut port = start_port;
    loop {
        if port_available(port).await {
            break;
        }
        port += 1;
    }
    port
}

///
/// Runs the web server.
///
pub async fn run_server(port: u16, state: AppState) -> std::io::Result<()> {
    println!("Starting server on http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
            // Session middleware: In production replace `[0; 32]` with a secure key and use HTTPS.
			.wrap(SessionMiddleware::new(
				CookieSessionStore::default(),
				// The Key should be 64 bytes. For development, you can use a simple key.
				Key::from(&[0; 64])
			))
            .service(Files::new("/static", "./static").show_files_listing())
            .service(index)
            .service(login)
            .service(metrics)
            .service(remotes)
            .service(get_pylons)
            .service(add_pylon)
            .service(remove_pylon)
            .service(admin_content)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
