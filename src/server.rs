// src/server.rs

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger, HttpRequest};
use std::sync::{Arc, Mutex, RwLock};
use crate::system_info::SystemData;
use crate::remote::RemoteStatus;
use serde_json::json;
use std::collections::HashMap;

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
      overflow-y: auto;
      max-height: 100vh;
      padding-right: 10px;
    }
    .card {
      background-color: #1e1e1e;
      padding: 20px;
      margin: 10px 0;
      border-radius: 8px;
      box-shadow: 0 2px 5px rgba(0,0,0,0.5);
    }
    /* Gauges */
    .gauges {
      display: flex;
      justify-content: space-around;
      flex-wrap: wrap;
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
    /* Flashing Globe Animation for Remote Poll Indicator */
    @keyframes pulse {
      0% { opacity: 1; }
      50% { opacity: 0.6; }
      100% { opacity: 1; }
    }
    .pulse {
      animation: pulse 2s ease-in-out infinite;
    }
  </style>
  <!-- Include ProgressBar.js and Chart.js -->
  <script src="https://cdnjs.cloudflare.com/ajax/libs/progressbar.js/1.0.1/progressbar.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
  <div class="container">
    <h1 id="localPylonName">Pylon Dashboard</h1>
    
    <!-- Local System Metrics Card -->
    <div class="card" id="localMetricsCard">
      <h2>Local System Metrics 💻</h2>
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
      <div style="margin-top: 20px;">
        <h3>Other Metrics ⏱️</h3>
        <div>Uptime: <span id="uptime">0</span> seconds</div>
        <div>Load Average: <span id="loadAverage">0</span></div>
      </div>
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
    
    <!-- Remote Pylons Card -->
    <div class="card" id="remoteCard">
      <h2>Remote Pylons <span id="remotePollIndicator"></span></h2>
      <div id="remoteContainer" style="max-height:300px; overflow-y:auto;"></div>
    </div>
    
    <!-- Remote Viewer Card -->
    <div class="card" id="remoteViewerCard">
      <h2>Remote Viewer 👀</h2>
      <div id="remoteView">
        <div><strong>⚡ CPU:</strong> <span id="rvCpu">0%</span></div>
        <div><strong>📊 RAM:</strong> <span id="rvRam">0%</span></div>
        <div><strong>💾 Disk:</strong> <span id="rvDisk">0%</span></div>
      </div>
    </div>
  </div>
  
  <!-- Inline JavaScript: Run after DOM is loaded -->
  <script>
  document.addEventListener('DOMContentLoaded', function() {
    // ----------------------
    // Global Variables
    // ----------------------
    var remoteGauges = {};  // Will hold gauges for each remote, keyed by safeKey.
    
    // ----------------------
    // Gauge Initialization for Local Metrics
    // ----------------------
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
    
    // ----------------------
    // Network Throughput Chart Setup
    // ----------------------
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
    
    // ----------------------
    // Fetch and Update Local Metrics & System Info
    // ----------------------
    async function fetchLocalMetrics() {
      try {
        let response = await fetch('/api/metrics');
        let data = await response.json();
        let polled = data.polled;
        let cached = data.cached;
        let localName = data.name;
        
        document.getElementById('localPylonName').innerText = localName + " 🚀✨";
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
        
        document.getElementById('uptime').innerText = polled.uptime;
        document.getElementById('loadAverage').innerText =
          polled.load_average.one + ' (1m), ' +
          polled.load_average.five + ' (5m), ' +
          polled.load_average.fifteen + ' (15m)';
        
        let newReceived = polled.network_received;
        let newTransmitted = polled.network_transmitted;
        if (previousReceived !== null && previousTransmitted !== null) {
          let deltaReceived = newReceived - previousReceived;
          let deltaTransmitted = newTransmitted - previousTransmitted;
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
                          "<tr><td>🔧 Processor</td><td>" + cached.processor + "</td></tr>" +
                          "<tr><td>💾 Total RAM</td><td>" + totalRamGB + " GB</td></tr>" +
                          "<tr><td>💿 Disk Capacity</td><td>" + totalDiskGB + " GB</td></tr>" +
                          "<tr><td>📀 Disk Usage</td><td>" + usedDiskGB + " GB</td></tr>" +
                          "<tr><td>⏰ Boot Time</td><td>" + bootDate + "</td></tr>" +
                          "</table>";
        document.getElementById('systemDetails').innerHTML = detailsHTML;
        
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
        
        document.getElementById('rvCpu').innerText = (polled.cpu_usage).toFixed(2) + "%";
        document.getElementById('rvRam').innerText = (ramPercent * 100).toFixed(2) + "%";
        document.getElementById('rvDisk').innerText = (diskPercent * 100).toFixed(2) + "%";
        
      } catch (err) {
        console.error('Error fetching local metrics:', err);
      }
    }
    
    // ----------------------
    // Fetch and Update Remote Pylon Metrics (Persistent Remote Blocks & Gauges)
    // ----------------------
    async function fetchRemoteMetrics() {
      try {
        let indicator = document.getElementById('remotePollIndicator');
        indicator.innerHTML = '<span class="pulse">🌐</span>';
        setTimeout(() => { indicator.innerHTML = ''; }, 1500);
        
        let response = await fetch('/api/remotes');
        let remotes = await response.json();
        let container = document.getElementById('remoteContainer');
        
        remotes.forEach(remote => {
          let safeKey = remote.ip.replace(/\./g, '_') + "_" + remote.port;
          let remoteBlock = document.getElementById('remote_' + safeKey);
          if (!remoteBlock) {
            // Create a new remote block if it doesn't exist
            remoteBlock = document.createElement('div');
            remoteBlock.id = 'remote_' + safeKey;
            remoteBlock.style.border = '1px solid #444';
            remoteBlock.style.padding = '10px';
            remoteBlock.style.marginBottom = '10px';
            remoteBlock.style.borderRadius = '5px';
            remoteBlock.style.backgroundColor = '#2a2a2a';
            
            let header = document.createElement('h3');
            header.style.margin = '0 0 10px';
            let remoteDisplayName = (remote.data && remote.data.name) ? remote.data.name : (remote.ip + ':' + remote.port);
            header.innerText = "🌍 " + remoteDisplayName;
            remoteBlock.appendChild(header);
            
            let gaugesContainer = document.createElement('div');
            gaugesContainer.id = 'gauges_' + safeKey;
            gaugesContainer.style.display = "flex";
            gaugesContainer.style.flexWrap = "wrap";
            gaugesContainer.style.gap = "10px";
            remoteBlock.appendChild(gaugesContainer);
            
            // CPU Gauge
            let cpuGaugeContainer = document.createElement('div');
            cpuGaugeContainer.className = "gauge-container";
            cpuGaugeContainer.style.width = "120px";
            let cpuGaugeDiv = document.createElement('div');
            let cpuGaugeId = 'cpuGauge_' + safeKey;
            cpuGaugeDiv.id = cpuGaugeId;
            cpuGaugeDiv.className = "gauge";
            cpuGaugeDiv.style.width = "100px";
            cpuGaugeDiv.style.height = "100px";
            cpuGaugeContainer.appendChild(cpuGaugeDiv);
            let cpuLabel = document.createElement('div');
            cpuLabel.className = "gauge-label";
            cpuLabel.innerText = "⚡ CPU";
            cpuGaugeContainer.appendChild(cpuLabel);
            gaugesContainer.appendChild(cpuGaugeContainer);
            
            // RAM Gauge
            let ramGaugeContainer = document.createElement('div');
            ramGaugeContainer.className = "gauge-container";
            ramGaugeContainer.style.width = "120px";
            let ramGaugeDiv = document.createElement('div');
            let ramGaugeId = 'ramGauge_' + safeKey;
            ramGaugeDiv.id = ramGaugeId;
            ramGaugeDiv.className = "gauge";
            ramGaugeDiv.style.width = "100px";
            ramGaugeDiv.style.height = "100px";
            ramGaugeContainer.appendChild(ramGaugeDiv);
            let ramLabel = document.createElement('div');
            ramLabel.className = "gauge-label";
            ramLabel.innerText = "📊 RAM";
            ramGaugeContainer.appendChild(ramLabel);
            let ramText = document.createElement('div');
            ramText.id = 'ramText_' + safeKey;
            ramText.style.fontSize = "0.9rem";
            ramGaugeContainer.appendChild(ramText);
            gaugesContainer.appendChild(ramGaugeContainer);
            
            // Disk Gauge
            let diskGaugeContainer = document.createElement('div');
            diskGaugeContainer.className = "gauge-container";
            diskGaugeContainer.style.width = "120px";
            let diskGaugeDiv = document.createElement('div');
            let diskGaugeId = 'diskGauge_' + safeKey;
            diskGaugeDiv.id = diskGaugeId;
            diskGaugeDiv.className = "gauge";
            diskGaugeDiv.style.width = "100px";
            diskGaugeDiv.style.height = "100px";
            diskGaugeContainer.appendChild(diskGaugeDiv);
            let diskLabel = document.createElement('div');
            diskLabel.className = "gauge-label";
            diskLabel.innerText = "💾 Disk";
            diskGaugeContainer.appendChild(diskLabel);
            let diskText = document.createElement('div');
            diskText.id = 'diskText_' + safeKey;
            diskText.style.fontSize = "0.9rem";
            diskGaugeContainer.appendChild(diskText);
            gaugesContainer.appendChild(diskGaugeContainer);
            
            // Create container for remote services status lights
            let remoteServices = document.createElement('div');
            remoteServices.id = 'remoteServices_' + safeKey;
            remoteServices.style.display = 'flex';
            remoteServices.style.justifyContent = 'center';
            remoteServices.style.gap = '10px';
            remoteServices.style.marginTop = '10px';
            remoteBlock.appendChild(remoteServices);
            
            container.appendChild(remoteBlock);
            
            // Initialize gauges for remote
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
          
          // Update the gauges if the remote is online.
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
            
            // Update remote services status lights
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
            // If offline, show an offline indicator with pylon name and flashing computer with red x.
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
  });
  </script>
</body>
</html>"#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[get("/api/metrics")]
async fn metrics(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let sys_data = data.system_data.lock().unwrap();
    let config = data.config.read().unwrap();
    let local_name = config.name.clone().unwrap_or_else(|| "Local Pylon".to_string());
    let response = json!({
        "name": local_name,
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
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
