// src/server.rs

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, middleware::Logger};
use actix_session::{Session, SessionMiddleware};
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use mime_guess::from_path;
use rust_embed::RustEmbed;

// These modules are assumed to be defined elsewhere in your project.
use crate::remote::RemoteStatus;
use crate::system_info::SystemData;

pub const PYLON_VERSION: &str = "0.2.3";


/// The shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<crate::config_manager::Config>>,
    pub system_data: Arc<Mutex<SystemData>>,
    pub remote_statuses: Arc<Mutex<HashMap<String, RemoteStatus>>>,
}

/// Embed the contents of the `static/` folder into the binary.
#[derive(RustEmbed)]
#[folder = "static/"] // Ensure this path is relative to your Cargo.toml location.
struct Asset;

/// Handler to serve embedded static files.
///
/// This route matches URLs of the form `/static/{filename:.*}`.
async fn serve_embedded_file(req: HttpRequest) -> impl Responder {
    // Extract the requested file path from the URL.
    let filename: String = req.match_info().query("filename").parse().unwrap_or_default();

    // Look up the file in the embedded assets.
    match Asset::get(&filename) {
        Some(content) => {
            // Guess the MIME type based on the file extension.
            let mime_type = from_path(&filename).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(content.data)
        },
        None => HttpResponse::NotFound().body("Not Found"),
    }
}

// This endpoint triggers an update check when called.
#[get("/api/check_update")]
async fn check_update_endpoint(data: web::Data<AppState>) -> impl Responder {
    // Call the updater's check_for_update function.
    match crate::updater::check_for_update(Arc::clone(&data.config)).await {
        Ok(updated) => {
            if updated {
                HttpResponse::Ok().json(json!({"status": "updated"}))
            } else {
                HttpResponse::Ok().json(json!({"status": "up-to-date"}))
            }
        }
        Err(e) => {
            // Fail silently by logging the error and returning an OK response.
            println!("Error during update check: {}", e);
            HttpResponse::Ok().json(json!({"status": "error", "message": e.to_string()}))
        }
    }
}

/// GET /
///
/// Serves the main HTML page.
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
  <link rel="icon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png">
  <link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png">
  <link rel="icon" type="image/png" sizes="192x192" href="/static/android-chrome-192x192.png">
  <link rel="icon" type="image/png" sizes="512x512" href="/static/android-chrome-512x512.png">
  <link rel="shortcut icon" href="/static/favicon.ico">
  <link rel="apple-touch-icon" href="/static/apple-touch-icon.png">
  <meta name="theme-color" content="black" /> 
  <meta name="msapplication-TileColor" content="black" />
  <script src="/static/js/progressbar.min.js"></script>
  <script src="/static/js/chart.js"></script>
</head>
<body>
  <div class="container">
    <h1 id="localPylonName">Pylon Dashboard</h1>
    <div class="card" id="localMetricsCard">
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
      <div class="services" id="servicesStatus"></div>
    </div>
    <h2>Remote Pylons 🌐</h2>
    <div id="remoteContainer"></div>
    <div class="card" id="networkCard">
      <h2>Network Throughput 🌐</h2>
      <div class="network-chart-container">
        <canvas id="networkChart"></canvas>
      </div>
    </div>
    <div class="card" id="adminLoginCard">
      <h2>Admin Access</h2>
      <input type="password" id="adminKeyInput" class="adminInput" placeholder="Enter Admin Key">
      <button id="adminKeySubmit" class="adminSubmit">Unlock Admin Features</button>
      <p id="adminError" style="color: red; display: none;">Incorrect key.</p>
    </div>
    <div id="adminContent"></div>
  </div>
  <script type="module" src="/static/js/main.js"></script>
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

/// POST /api/login
///
/// Accepts a JSON payload with the admin token and, if valid, marks the session as authenticated.
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

/// GET /api/admin-content
///
/// Returns admin-only HTML if the session is authenticated.
#[get("/api/admin-content")]
async fn admin_content(session: Session) -> impl Responder {
    if let Ok(Some(true)) = session.get::<bool>("admin_authenticated") {
        let admin_html = r#"
      <div class="card" id="systemInfoCard">
        <h2>System Information 📋</h2>
        <div id="systemDetails"></div>
      </div>
      <div class="card" id="otherMetricsCard">
        <h2>Other Metrics ⏱️</h2>
        <div>Uptime: <span id="uptime">0</span> seconds</div>
        <div>Load Average: <span id="loadAverage">0</span></div>
      </div>
      <div class="card" id="topProcessesCard">
        <h2>Top 5 Processes by Memory</h2>
        <table id="topProcessesTable">
          <thead>
            <tr><th>PID</th><th>Name</th><th>Memory (MB)</th></tr>
          </thead>
          <tbody></tbody>
        </table>
      </div>
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

/// GET /api/metrics
///
/// Returns local metrics as JSON.
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

/// GET /api/remotes
///
/// Returns remote pylon statuses as JSON.
#[get("/api/remotes")]
async fn remotes(data: web::Data<AppState>) -> impl Responder {
    let statuses = data.remote_statuses.lock().unwrap();
    let response: Vec<_> = statuses.values().cloned().collect();
    HttpResponse::Ok().json(response)
}

/// GET /api/config/pylons
///
/// Returns the current remote pylons configuration.
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

/// POST /api/config/pylons/add
///
/// Adds a new remote pylon to the configuration.
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

/// POST /api/config/pylons/remove
///
/// Removes a remote pylon from the configuration.
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

/// Checks if a port is available.
async fn port_available(port: u16) -> bool {
    use tokio::net::TcpListener;
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Finds an open port starting from `start_port`.
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

/// Runs the web server.
pub async fn run_server(port: u16, state: AppState) -> std::io::Result<()> {
    println!("Starting server on http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(&[0; 64]) // In production, use a secure key!
            ))
            // Route to serve embedded static files.
            .route("/static/{filename:.*}", web::get().to(serve_embedded_file))
            .service(index)
            .service(login)
            .service(metrics)
            .service(remotes)
            .service(get_pylons)
            .service(add_pylon)
            .service(remove_pylon)
            .service(admin_content)
			.service(check_update_endpoint)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
