// src/server.rs

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger, HttpRequest};
use std::sync::{Arc, Mutex, RwLock};
use crate::system_info::SystemData;
use crate::remote::RemoteStatus;
use serde_json::json;
use std::collections::HashMap;
use serde::Deserialize;

const PYLON_VERSION: &str = "0.1.2";

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<crate::config_manager::Config>>,
    pub system_data: Arc<Mutex<SystemData>>,
    pub remote_statuses: Arc<Mutex<HashMap<String, RemoteStatus>>>,
}

#[get("/")]
async fn index() -> impl Responder {
    let html = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>Pylon Dashboard 🚀✨</title>
  <style>
    /* Global Styles */
    body {
      background-color: #121212;
      color: #e0e0e0;
      font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
      margin: 0;
      padding: 0;
    }
    h1, h2, h3 {
      text-shadow: 0 0 5px rgba(0,255,255,0.5);
    }
    .container {
      width: 90%;
      margin: 20px auto;
      padding-right: 10px;
    }
    .card {
      background-color: #1e1e1e;
      padding: 20px;
      margin: 10px 0;
      border-radius: 8px;
      box-shadow: 0 2px 5px rgba(0,0,0,0.5);
    }
    /* Hover effect for clickable pylon cards */
    .pylon-card:hover {
      cursor: pointer;
      box-shadow: 0 0 10px rgba(0,255,255,0.5);
      transition: box-shadow 0.3s;
    }
    /* Gauges: always in one horizontal row */
    .gauges {
      display: flex;
      justify-content: space-around;
      /* Never wrap – allow horizontal scrolling if needed */
      flex-wrap: nowrap;
      overflow-x: auto;
    }
    .gauge-container {
      text-align: center;
      margin: 20px;
      padding: 10px;
      border-radius: 10px;
    }
    .gauge {
      width: 180px;
      height: 180px;
      margin: auto;
      filter: drop-shadow(0 0 10px rgba(0, 255, 255, 0.7));
    }
    .gauge-label {
      margin-top: 8px;
      font-size: 1.2rem;
    }
    /* Local Services Status */
    .services {
      display: flex;
      justify-content: center;
      gap: 20px;
      margin-top: 20px;
    }
    .service {
      text-align: center;
    }
    .service-light {
      width: 20px;
      height: 20px;
      border-radius: 50%;
      margin: 0 auto;
      box-shadow: 0 0 5px;
    }
    /* Network Chart */
    .network-chart-container {
      position: relative;
      height: 300px;
      width: 100%;
    }
    #networkChart {
      width: 100% !important;
      height: 100% !important;
      background-color: #1e1e1e;
      border-radius: 8px;
      display: block;
    }
    /* Table Styling */
    table {
      width: 100%;
      border-collapse: collapse;
      margin-top: 10px;
    }
    table, th, td {
      border: 1px solid #333;
    }
    th, td {
      padding: 8px;
      text-align: left;
    }
    th {
      background-color: #222;
    }
    /* Scrollbar Styling */
    ::-webkit-scrollbar {
      width: 8px;
      height: 8px;
    }
    ::-webkit-scrollbar-track {
      background: #1e1e1e;
    }
    ::-webkit-scrollbar-thumb {
      background: #333;
      border-radius: 4px;
    }
    /* Flashing globe animation */
    @keyframes pulse {
      0% { opacity: 1; }
      50% { opacity: 0.6; }
      100% { opacity: 1; }
    }
    .pulse {
      animation: pulse 2s ease-in-out infinite;
    }
    /* Form Styles for Manage Remote Pylons */
    #pylonForm input, #pylonForm button {
      padding: 8px;
      margin: 5px;
      border: none;
      border-radius: 4px;
    }
    #pylonForm input {
      width: calc(25% - 12px);
    }
    #pylonForm button {
      background-color: #2196F3;
      color: #fff;
      cursor: pointer;
    }
    #pylonList {
      list-style-type: none;
      padding: 0;
    }
    #pylonList li {
      padding: 5px;
      border-bottom: 1px solid #444;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    #pylonList button {
      background-color: #f44336;
      border: none;
      color: #fff;
      padding: 5px 10px;
      border-radius: 4px;
      cursor: pointer;
    }
    /* Modal styles */
    .modal {
      display: none;
      position: fixed;
      z-index: 1000;
      left: 0;
      top: 0;
      width: 100%;
      height: 100%;
      overflow: auto;
      background-color: rgba(0,0,0,0.6);
    }
    .modal-content {
      background-color: #1e1e1e;
      margin: 15% auto;
      padding: 20px;
      border: 1px solid #888;
      width: 80%;
      max-width: 500px;
      border-radius: 10px;
      text-align: center;
      font-size: 1.2rem;
    }
    .close {
      color: #aaa;
      float: right;
      font-size: 28px;
      font-weight: bold;
      cursor: pointer;
    }
    .close:hover,
    .close:focus {
      color: #fff;
      text-decoration: none;
      cursor: pointer;
    }
    /* Pylon header texts */
    .pylon-server-name {
      font-size: 1.8rem;
      color: #e0e0e0;
    }
    .pylon-location {
      font-size: 1rem;
      color: #9e9e9e;
    }
    .pylon-version {
      font-size: 1rem;
      color: #b0bec5;
    }
    /* Mobile-specific adjustments */
    @media (max-width: 600px) {
      .container {
        width: 95%;
        margin: 10px auto;
      }
      .gauges {
        flex-direction: row;
        flex-wrap: nowrap;
        overflow-x: auto;
        justify-content: flex-start;
      }
    }
  </style>
  <!-- Include ProgressBar.js and Chart.js -->
  <script src="https://cdnjs.cloudflare.com/ajax/libs/progressbar.js/1.0.1/progressbar.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
  <div class="container">
    <!-- The pylon header (name, location, version) -->
    <h1 id="localPylonName">Pylon Dashboard</h1>
    
    <!-- Local Pylon Card: note the header text has been removed -->
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
    <!-- Remote Pylons Container: each remote gets its own full–width card -->
    <div id="remoteContainer">
      <!-- Remote pylon cards are appended here dynamically -->
    </div>
    
    <!-- Network Throughput Card -->
    <div class="card" id="networkCard">
      <h2>Network Throughput 🌐</h2>
      <div class="network-chart-container">
        <canvas id="networkChart"></canvas>
      </div>
    </div>
    
    <!-- System Information Card -->
    <div class="card" id="systemInfoCard">
      <h2>System Information 📋</h2>
      <div id="systemDetails">
        <!-- Filled dynamically -->
      </div>
    </div>
    
    <!-- New Other Metrics Card (moved from the local card) -->
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
        <tbody>
          <!-- Filled dynamically -->
        </tbody>
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
  </div>
  
  <!-- Inline JavaScript (runs after DOM load) -->
  <script>
    function showModal(description) {
      let modal = document.getElementById("pylonModal");
      let modalDescription = document.getElementById("modalDescription");
      modalDescription.innerText = description;
      modal.style.display = "block";
    }
    function hideModal() {
      document.getElementById("pylonModal").style.display = "none";
    }
    document.addEventListener('DOMContentLoaded', function() {
      // Modal close events
      document.querySelector(".close").addEventListener("click", hideModal);
      window.addEventListener("click", function(event) {
        let modal = document.getElementById("pylonModal");
        if (event.target == modal) {
          hideModal();
        }
      });
      
      // Make the local metrics card clickable:
      let localCard = document.getElementById("localMetricsCard");
      localCard.classList.add("pylon-card");
      localCard.addEventListener("click", function() {
        let desc = localCard.getAttribute("data-description") || "Sorry, no description was provided for this Pylon.";
        showModal(desc);
      });
      
      // Global variable for remote gauges
      var remoteGauges = {};
      
      // Initialize local gauges
      var cpuGauge = new ProgressBar.Circle('#cpuGauge', {
        color: '#00ff00',
        strokeWidth: 6,
        trailWidth: 2,
        easing: 'easeInOut',
        duration: 800,
        text: { value: '0.00%' },
        from: { color: '#ff4444', width: 2 },
        to: { color: '#00ff00', width: 6 },
        step: function(state, circle) {
          var value = (circle.value() * 100).toFixed(2);
          circle.path.setAttribute('stroke', state.color);
          circle.path.setAttribute('stroke-width', state.width);
          circle.setText(value + '%');
        }
      });
      cpuGauge.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
      cpuGauge.text.style.fontSize = '1.5rem';
      
      var ramGauge = new ProgressBar.Circle('#ramGauge', {
        color: '#2196F3',
        strokeWidth: 6,
        trailWidth: 2,
        easing: 'easeInOut',
        duration: 800,
        text: { value: '0.00%' },
        from: { color: '#ff5722', width: 2 },
        to: { color: '#2196F3', width: 6 },
        step: function(state, circle) {
          var value = (circle.value() * 100).toFixed(2);
          circle.path.setAttribute('stroke', state.color);
          circle.path.setAttribute('stroke-width', state.width);
          circle.setText(value + '%');
        }
      });
      ramGauge.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
      ramGauge.text.style.fontSize = '1.5rem';
      
      var diskGauge = new ProgressBar.Circle('#diskGauge', {
        color: '#ffcc00',
        strokeWidth: 6,
        trailWidth: 2,
        easing: 'easeInOut',
        duration: 800,
        text: { value: '0.00%' },
        from: { color: '#ff4444', width: 2 },
        to: { color: '#ffcc00', width: 6 },
        step: function(state, circle) {
          var value = (circle.value() * 100).toFixed(2);
          circle.path.setAttribute('stroke', state.color);
          circle.path.setAttribute('stroke-width', state.width);
          circle.setText(value + '%');
        }
      });
      diskGauge.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
      diskGauge.text.style.fontSize = '1.5rem';
      
      // Network Throughput Chart Setup
      const ctx = document.getElementById('networkChart').getContext('2d');
      const networkChart = new Chart(ctx, {
        type: 'line',
        data: {
          labels: [],
          datasets: [
            {
              label: '📥 Received (KB/s)',
              data: [],
              borderColor: 'orange',
              backgroundColor: 'rgba(255,165,0,0.2)',
              fill: false,
              tension: 0.1
            },
            {
              label: '📤 Transmitted (KB/s)',
              data: [],
              borderColor: 'cyan',
              backgroundColor: 'rgba(0,255,255,0.2)',
              fill: false,
              tension: 0.1
            }
          ]
        },
        options: {
          scales: {
            x: { title: { display: true, text: 'Time (s)' } },
            y: { title: { display: true, text: 'KB/s' }, beginAtZero: true }
          },
          animation: false,
          responsive: true,
          maintainAspectRatio: false,
        }
      });
      
      var previousReceived = null;
      var previousTransmitted = null;
      var startTime = Date.now();
      const maxDataPoints = 30;
      
      function updateNetworkChart(receivedKBs, transmittedKBs) {
        let nowSec = Math.floor((Date.now() - startTime) / 1000);
        networkChart.data.labels.push(nowSec);
        networkChart.data.datasets[0].data.push(receivedKBs);
        networkChart.data.datasets[1].data.push(transmittedKBs);
        if (networkChart.data.labels.length > maxDataPoints) {
          networkChart.data.labels.shift();
          networkChart.data.datasets[0].data.shift();
          networkChart.data.datasets[1].data.shift();
        }
        networkChart.update();
      }
      
      async function fetchLocalMetrics() {
        try {
          let response = await fetch('/api/metrics');
          let data = await response.json();
          let polled = data.polled;
          let cached = data.cached;
          // Update the pylon header with name, location, version.
          document.getElementById('localPylonName').innerHTML =
            `<span class="pylon-server-name">${data.name}</span> <span class="pylon-location">(${data.location})</span> 🚀✨ <span class="pylon-version">(v${data.version})</span>`;
          
          cpuGauge.animate(polled.cpu_usage / 100);
          let totalRam = cached.total_ram;
          let usedRam = polled.used_ram;
          let ramPercent = totalRam > 0 ? usedRam / totalRam : 0;
          ramGauge.animate(ramPercent);
          let totalRamGB = (totalRam / 1024 / 1024).toFixed(2);
          let usedRamGB = (usedRam / 1024 / 1024).toFixed(2);
          document.getElementById('ramUsageText').innerText = usedRamGB + " GB / " + totalRamGB + " GB used";
          
          let totalDisk = cached.disk_capacity;
          let usedDisk = totalDisk > 0 ? (totalDisk - polled.disk_free) : 0;
          let diskPercent = totalDisk > 0 ? usedDisk / totalDisk : 0;
          diskGauge.animate(diskPercent);
          let totalDiskGB = (totalDisk / (1024*1024*1024)).toFixed(2);
          let usedDiskGB = (usedDisk / (1024*1024*1024)).toFixed(2);
          document.getElementById('diskUsageText').innerText = usedDiskGB + " GB / " + totalDiskGB + " GB used";
          document.getElementById('localMetricsCard').setAttribute("data-description",
            data.description || "Sorry, no description was provided for this Pylon.");
          
          // NETWORK THROUGHPUT: Compute delta but ignore negative differences.
          let newReceived = polled.network_received;
          let newTransmitted = polled.network_transmitted;
          if (previousReceived !== null && previousTransmitted !== null) {
            let deltaReceived = newReceived - previousReceived;
            let deltaTransmitted = newTransmitted - previousTransmitted;
            // Prevent negative deltas (which might occur if counters reset)
            if(deltaReceived < 0) { deltaReceived = 0; }
            if(deltaTransmitted < 0) { deltaTransmitted = 0; }
            let throughputReceived = (deltaReceived / 1) / 1024;
            let throughputTransmitted = (deltaTransmitted / 1) / 1024;
            updateNetworkChart(throughputReceived.toFixed(2), throughputTransmitted.toFixed(2));
          }
          previousReceived = newReceived;
          previousTransmitted = newTransmitted;
          
          let bootDate = new Date(cached.boot_time * 1000).toLocaleString();
          let detailsHTML = "<table>" +
                            "<tr><th>Property</th><th>Value</th></tr>" +
                            "<tr><td>🌟 OS Version</td><td>" + cached.os_version + "</td></tr>" +
                            "<tr><td>⚙️ Apache Version</td><td>" + cached.apache_version + "</td></tr>" +
                            "<tr><td>🐘 PHP Version</td><td>" + cached.php_version + "</td></tr>" +
                            "<tr><td>💽 MariaDB Version</td><td>" + cached.mariadb_version + "</td></tr>" +
                            "<tr><td>🦀 Rust Version</td><td>" + cached.rust_version + "</td></tr>" +
                            "<tr><td>📟 Node Version</td><td>" + cached.node_version + "</td></tr>" +
                            "<tr><td>📦 npm Version</td><td>" + cached.npm_version + "</td></tr>" +
                            "<tr><td>🚀 Pylon Version</td><td>" + data.version + "</td></tr>" +
                            "<tr><td>🔧 Processor</td><td>" + cached.processor + "</td></tr>" +
                            "<tr><td>💾 Total RAM</td><td>" + totalRamGB + " GB</td></tr>" +
                            "<tr><td>💿 Disk Capacity</td><td>" + totalDiskGB + " GB</td></tr>" +
                            "<tr><td>📀 Disk Usage</td><td>" + usedDiskGB + " GB</td></tr>" +
                            "<tr><td>⏰ Boot Time</td><td>" + bootDate + "</td></tr>" +
                            "</table>";
          document.getElementById('systemDetails').innerHTML = detailsHTML;
          
          // Update the Other Metrics card (Uptime and Load Average)
          document.getElementById('uptime').innerText = polled.uptime;
          document.getElementById('loadAverage').innerText =
            polled.load_average.one + ' (1m), ' +
            polled.load_average.five + ' (5m), ' +
            polled.load_average.fifteen + ' (15m)';
          
          let topProcessesTableBody = document.getElementById('topProcessesTable').getElementsByTagName('tbody')[0];
          topProcessesTableBody.innerHTML = "";
          if (polled.top_processes && polled.top_processes.length > 0) {
            polled.top_processes.forEach(proc => {
              let row = document.createElement('tr');
              let pidCell = document.createElement('td');
              pidCell.innerText = proc.pid;
              let nameCell = document.createElement('td');
              nameCell.innerText = proc.name;
              let memCell = document.createElement('td');
              let memMB = (proc.memory / 1024).toFixed(2);
              memCell.innerText = memMB;
              row.appendChild(pidCell);
              row.appendChild(nameCell);
              row.appendChild(memCell);
              topProcessesTableBody.appendChild(row);
            });
          }
          
          let servicesDiv = document.getElementById('servicesStatus');
          servicesDiv.innerHTML = "";
          polled.services.forEach(service => {
            let serviceDiv = document.createElement('div');
            serviceDiv.className = "service";
            let light = document.createElement('div');
            light.className = "service-light";
            light.style.backgroundColor = service.running ? "limegreen" : "red";
            light.style.boxShadow = service.running ? "0 0 10px limegreen" : "0 0 10px red";
            let label = document.createElement('div');
            label.innerText = service.name;
            serviceDiv.appendChild(light);
            serviceDiv.appendChild(label);
            servicesDiv.appendChild(serviceDiv);
          });
        } catch (err) {
          console.error('Error fetching local metrics:', err);
        }
      }
      
      async function fetchRemoteMetrics() {
        try {
          // Optional remote poll indicator
          let indicator = document.getElementById('remotePollIndicator');
          if(indicator){
            indicator.innerHTML = '<span class="pulse">🌐</span>';
            setTimeout(() => { indicator.innerHTML = ''; }, 1500);
          }
          
          let response = await fetch('/api/remotes');
          let remotes = await response.json();
          let container = document.getElementById('remoteContainer');
          
          remotes.forEach(remote => {
            let safeKey = remote.ip.replace(/\./g, '_') + "_" + remote.port;
            let remoteBlock = document.getElementById('remote_' + safeKey);
            if (!remoteBlock) {
              remoteBlock = document.createElement('div');
              remoteBlock.id = 'remote_' + safeKey;
              remoteBlock.className = "card";
              remoteBlock.style.marginBottom = "10px";
              remoteBlock.setAttribute("data-description",
                remote.description ? remote.description : "Sorry, no description was provided for this Pylon.");
              remoteBlock.style.cursor = "pointer";
              remoteBlock.addEventListener("click", function() {
                showModal(this.getAttribute("data-description"));
              });
              
              let header = document.createElement('h3');
              header.style.margin = '0 0 10px';
              let remoteDisplayName = (remote.data && remote.data.name) ? remote.data.name : (remote.ip + ':' + remote.port);
              let remoteVersion = (remote.data && remote.data.version) ? remote.data.version : "unknown";
              let remoteLocation = remote.location || "Unknown Location";
              header.innerHTML = `🌍 <span class="pylon-server-name">${remoteDisplayName}</span> <span class="pylon-location">(${remoteLocation})</span> <span class="pylon-version">(v${remoteVersion})</span>`;
              remoteBlock.appendChild(header);
              
              let gaugesContainer = document.createElement('div');
              gaugesContainer.id = 'gauges_' + safeKey;
              gaugesContainer.style.display = "flex";
              gaugesContainer.style.flexDirection = "row";
              gaugesContainer.style.justifyContent = "space-around";
              gaugesContainer.style.overflowX = "auto";
              gaugesContainer.style.gap = "10px";
              remoteBlock.appendChild(gaugesContainer);
              
              // CPU Gauge
              let cpuGaugeContainer = document.createElement('div');
              cpuGaugeContainer.className = "gauge-container";
              let cpuGaugeDiv = document.createElement('div');
              let cpuGaugeId = 'cpuGauge_' + safeKey;
              cpuGaugeDiv.id = cpuGaugeId;
              cpuGaugeDiv.className = "gauge";
              cpuGaugeContainer.appendChild(cpuGaugeDiv);
              let cpuLabel = document.createElement('div');
              cpuLabel.className = "gauge-label";
              cpuLabel.innerText = "⚡ CPU";
              cpuGaugeContainer.appendChild(cpuLabel);
              gaugesContainer.appendChild(cpuGaugeContainer);
              
              // RAM Gauge
              let ramGaugeContainer = document.createElement('div');
              ramGaugeContainer.className = "gauge-container";
              let ramGaugeDiv = document.createElement('div');
              let ramGaugeId = 'ramGauge_' + safeKey;
              ramGaugeDiv.id = ramGaugeId;
              ramGaugeDiv.className = "gauge";
              ramGaugeContainer.appendChild(ramGaugeDiv);
              let ramLabel = document.createElement('div');
              ramLabel.className = "gauge-label";
              ramLabel.innerText = "📊 RAM";
              ramGaugeContainer.appendChild(ramLabel);
              let ramText = document.createElement('div');
              ramText.id = 'ramText_' + safeKey;
              ramText.style.fontSize = "1rem";
              ramGaugeContainer.appendChild(ramText);
              gaugesContainer.appendChild(ramGaugeContainer);
              
              // Disk Gauge
              let diskGaugeContainer = document.createElement('div');
              diskGaugeContainer.className = "gauge-container";
              let diskGaugeDiv = document.createElement('div');
              let diskGaugeId = 'diskGauge_' + safeKey;
              diskGaugeDiv.id = diskGaugeId;
              diskGaugeDiv.className = "gauge";
              diskGaugeContainer.appendChild(diskGaugeDiv);
              let diskLabel = document.createElement('div');
              diskLabel.className = "gauge-label";
              diskLabel.innerText = "💾 Disk";
              diskGaugeContainer.appendChild(diskLabel);
              let diskText = document.createElement('div');
              diskText.id = 'diskText_' + safeKey;
              diskText.style.fontSize = "1rem";
              diskGaugeContainer.appendChild(diskText);
              gaugesContainer.appendChild(diskGaugeContainer);
              
              // Remote Services Status Lights Container
              let remoteServices = document.createElement('div');
              remoteServices.id = 'remoteServices_' + safeKey;
              remoteServices.style.display = 'flex';
              remoteServices.style.justifyContent = 'center';
              remoteServices.style.gap = '10px';
              remoteServices.style.marginTop = '10px';
              remoteBlock.appendChild(remoteServices);
              
              container.appendChild(remoteBlock);
              
              remoteGauges[safeKey] = {
                cpu: new ProgressBar.Circle('#' + cpuGaugeId, {
                  color: '#00ff00',
                  strokeWidth: 4,
                  trailWidth: 2,
                  easing: 'easeInOut',
                  duration: 800,
                  text: { value: '0.00%' },
                  from: { color: '#ff4444', width: 2 },
                  to: { color: '#00ff00', width: 4 },
                  step: function(state, circle) {
                    var value = (circle.value() * 100).toFixed(2);
                    circle.path.setAttribute('stroke', state.color);
                    circle.path.setAttribute('stroke-width', state.width);
                    circle.setText(value + '%');
                  }
                }),
                ram: new ProgressBar.Circle('#' + ramGaugeId, {
                  color: '#2196F3',
                  strokeWidth: 4,
                  trailWidth: 2,
                  easing: 'easeInOut',
                  duration: 800,
                  text: { value: '0.00%' },
                  from: { color: '#ff5722', width: 2 },
                  to: { color: '#2196F3', width: 4 },
                  step: function(state, circle) {
                    var value = (circle.value() * 100).toFixed(2);
                    circle.path.setAttribute('stroke', state.color);
                    circle.path.setAttribute('stroke-width', state.width);
                    circle.setText(value + '%');
                  }
                }),
                disk: new ProgressBar.Circle('#' + diskGaugeId, {
                  color: '#ffcc00',
                  strokeWidth: 4,
                  trailWidth: 2,
                  easing: 'easeInOut',
                  duration: 800,
                  text: { value: '0.00%' },
                  from: { color: '#ff4444', width: 2 },
                  to: { color: '#ffcc00', width: 4 },
                  step: function(state, circle) {
                    var value = (circle.value() * 100).toFixed(2);
                    circle.path.setAttribute('stroke', state.color);
                    circle.path.setAttribute('stroke-width', state.width);
                    circle.setText(value + '%');
                  }
                })
              };
              remoteGauges[safeKey].cpu.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
              remoteGauges[safeKey].cpu.text.style.fontSize = '1rem';
              remoteGauges[safeKey].ram.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
              remoteGauges[safeKey].ram.text.style.fontSize = '1rem';
              remoteGauges[safeKey].disk.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
              remoteGauges[safeKey].disk.text.style.fontSize = '1rem';
            }
            
            if (remote.online && remote.data) {
              let polled = remote.data.polled;
              let cached = remote.data.cached;
              remoteGauges[safeKey].cpu.animate(polled.cpu_usage / 100);
              
              let totalRam = cached.total_ram;
              let usedRam = polled.used_ram;
              let ramPercent = totalRam > 0 ? usedRam / totalRam : 0;
              remoteGauges[safeKey].ram.animate(ramPercent);
              let totalRamGB = (totalRam / 1024 / 1024).toFixed(2);
              let usedRamGB = (usedRam / 1024 / 1024).toFixed(2);
              document.getElementById('ramText_' + safeKey).innerText = usedRamGB + " GB / " + totalRamGB + " GB";
              
              let totalDisk = cached.disk_capacity;
              let usedDisk = totalDisk > 0 ? (totalDisk - polled.disk_free) : 0;
              let diskPercent = totalDisk > 0 ? usedDisk / totalDisk : 0;
              remoteGauges[safeKey].disk.animate(diskPercent);
              let totalDiskGB = (totalDisk / (1024*1024*1024)).toFixed(2);
              let usedDiskGB = (usedDisk / (1024*1024*1024)).toFixed(2);
              document.getElementById('diskText_' + safeKey).innerText = usedDiskGB + " GB / " + totalDiskGB + " GB";
              
              let remoteServicesDiv = document.getElementById('remoteServices_' + safeKey);
              if (remoteServicesDiv) {
                remoteServicesDiv.innerHTML = "";
                if (polled.services) {
                  polled.services.forEach(service => {
                    let serviceDiv = document.createElement('div');
                    serviceDiv.className = "service";
                    let light = document.createElement('div');
                    light.className = "service-light";
                    light.style.backgroundColor = service.running ? "limegreen" : "red";
                    light.style.boxShadow = service.running ? "0 0 10px limegreen" : "0 0 10px red";
                    let label = document.createElement('div');
                    label.innerText = service.name;
                    serviceDiv.appendChild(light);
                    serviceDiv.appendChild(label);
                    remoteServicesDiv.appendChild(serviceDiv);
                  });
                }
              }
            } else {
              let displayName = remote.name ? remote.name : (remote.ip + ':' + remote.port);
              let remoteBlock = document.getElementById('remote_' + safeKey);
              remoteBlock.innerHTML = '<div style="font-size:1.5rem; text-align:center;">' + displayName + '<br><span class="pulse" style="color:red;">💻❌</span></div>';
            }
          });
        } catch (err) {
          console.error('Error fetching remote metrics:', err);
        }
      }
      
      async function updateDashboard() {
        await fetchLocalMetrics();
        await fetchRemoteMetrics();
      }
      
      setInterval(updateDashboard, 1000);
      updateDashboard();
      
      // Manage Remote Pylons Functionality
      async function fetchPylonConfig() {
        try {
          let response = await fetch('/api/config/pylons');
          let pylons = await response.json();
          let list = document.getElementById('pylonList');
          list.innerHTML = "";
          if (pylons && Array.isArray(pylons)) {
            pylons.forEach(pylon => {
              let li = document.createElement('li');
              li.innerText = pylon.ip + ":" + pylon.port + " (" + (pylon.name || "No Name") + ")";
              let removeBtn = document.createElement('button');
              removeBtn.innerText = "Remove";
              removeBtn.onclick = async function() {
                await fetch('/api/config/pylons/remove', {
                  method: 'POST',
                  headers: {'Content-Type': 'application/json'},
                  body: JSON.stringify({ ip: pylon.ip, port: pylon.port })
                });
                fetchPylonConfig();
              };
              li.appendChild(removeBtn);
              list.appendChild(li);
            });
          }
        } catch (err) {
          console.error('Error fetching pylon config:', err);
        }
      }
      
      document.getElementById('pylonForm').addEventListener('submit', async function(e) {
        e.preventDefault();
        let ip = document.getElementById('pylonIp').value;
        let port = parseInt(document.getElementById('pylonPort').value);
        let token = document.getElementById('pylonToken').value;
        let name = document.getElementById('pylonName').value;
        let newPylon = { ip, port, token, name: name || null };
        await fetch('/api/config/pylons/add', {
          method: 'POST',
          headers: {'Content-Type': 'application/json'},
          body: JSON.stringify(newPylon)
        });
        document.getElementById('pylonForm').reset();
        fetchPylonConfig();
      });
      
      fetchPylonConfig();
    });
  </script>
  
  <!-- Modal for Pylon Description -->
  <div id="pylonModal" class="modal">
    <div class="modal-content">
      <span class="close">&times;</span>
      <h2>Pylon Description</h2>
      <p id="modalDescription"></p>
    </div>
  </div>
</body>
</html>"#;
    HttpResponse::Ok().content_type("text/html").body(html)
}




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

#[get("/api/remotes")]
async fn remotes(data: web::Data<AppState>) -> impl Responder {
    let statuses = data.remote_statuses.lock().unwrap();
    let response: Vec<_> = statuses.values().cloned().collect();
    HttpResponse::Ok().json(response)
}

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

#[post("/api/config/pylons/add")]
async fn add_pylon(data: web::Data<AppState>, new_pylon: web::Json<crate::config_manager::RemotePylonConfig>) -> impl Responder {
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

#[post("/api/config/pylons/remove")]
async fn remove_pylon(data: web::Data<AppState>, info: web::Json<RemovePylonRequest>) -> impl Responder {
    let mut config = data.config.write().unwrap();
    if let Some(ref mut pylons) = config.remote_pylons {
        pylons.retain(|p| !(p.ip == info.ip && p.port == info.port));
    }
    match crate::config_manager::save_config(&config) {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "removed"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

async fn port_available(port: u16) -> bool {
    use tokio::net::TcpListener;
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

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

pub async fn run_server(port: u16, state: AppState) -> std::io::Result<()> {
    println!("Starting server on http://127.0.0.1:{}", port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(Logger::default())
            .service(index)
            .service(metrics)
            .service(remotes)
            .service(get_pylons)
            .service(add_pylon)
            .service(remove_pylon)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
