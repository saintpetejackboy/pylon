// /static/js/main.js

import { showModal, hideModal } from './modal.js';
import { loginAdmin, loadAdminContent } from './admin.js';
import { initGauges, updateNetworkChart, fetchLocalMetrics, fetchRemoteMetrics } from './dashboard.js';

document.addEventListener('DOMContentLoaded', function() {
  // ----- Modal Close Events -----
  const closeBtn = document.querySelector(".close");
  if (closeBtn) {
    closeBtn.addEventListener("click", hideModal);
  }
  window.addEventListener("click", function(event) {
    const modal = document.getElementById("pylonModal");
    if (event.target === modal) {
      hideModal();
    }
  });

  // ----- Local Metrics Card Click (Modal Trigger) -----
  const localCard = document.getElementById("localMetricsCard");
  if (localCard) {
    localCard.classList.add("pylon-card");
    localCard.addEventListener("click", function() {
      const desc = localCard.getAttribute("data-description") ||
                   "Sorry, no description was provided for this Pylon.";
      showModal(desc);
    });
  }

  // ----- Admin Login Functionality -----
  const adminKeySubmit = document.getElementById('adminKeySubmit');
  if (adminKeySubmit) {
    adminKeySubmit.addEventListener('click', loginAdmin);
  }
  // Auto–hide the login panel if already unlocked
  if (sessionStorage.getItem('adminUnlocked') === 'true') {
    document.getElementById('adminLoginCard').style.display = 'none';
  }

  // ----- Initialize Gauges and Chart -----
  const { cpuGauge, ramGauge, diskGauge, networkChart } = initGauges();
  const startTime = Date.now();
  const maxDataPoints = 30;
  let previousReceived = null;
  let previousTransmitted = null;
  
  // ----- Callback: Update Local Gauges and Network Chart -----
  function updateGaugesCallback(data) {
    // CPU Gauge
    cpuGauge.animate(data.polled.cpu_usage / 100);

    // RAM Gauge
    const totalRam = data.cached.total_ram;
    const usedRam = data.polled.used_ram;
    const ramPercent = totalRam > 0 ? usedRam / totalRam : 0;
    ramGauge.animate(ramPercent);
    const totalRamGB = (totalRam / 1024 / 1024).toFixed(2);
    const usedRamGB = (usedRam / 1024 / 1024).toFixed(2);
    const ramUsageText = document.getElementById('ramUsageText');
    if (ramUsageText) {
      ramUsageText.innerText = `${usedRamGB} GB / ${totalRamGB} GB used`;
    }

    // Disk Gauge
    const totalDisk = data.cached.disk_capacity;
    const usedDisk = totalDisk > 0 ? (totalDisk - data.polled.disk_free) : 0;
    const diskPercent = totalDisk > 0 ? usedDisk / totalDisk : 0;
    diskGauge.animate(diskPercent);
    const totalDiskGB = (totalDisk / (1024 * 1024 * 1024)).toFixed(2);
    const usedDiskGB = (usedDisk / (1024 * 1024 * 1024)).toFixed(2);
    const diskUsageText = document.getElementById('diskUsageText');
    if (diskUsageText) {
      diskUsageText.innerText = `${usedDiskGB} GB / ${totalDiskGB} GB used`;
    }

    // Update the local card description
    const localMetricsCard = document.getElementById('localMetricsCard');
    if (localMetricsCard) {
      localMetricsCard.setAttribute("data-description", data.description ||
        "Sorry, no description was provided for this Pylon.");
    }

    // Update the network throughput chart
    const newReceived = data.polled.network_received;
    const newTransmitted = data.polled.network_transmitted;
    if (previousReceived !== null && previousTransmitted !== null) {
      let deltaReceived = newReceived - previousReceived;
      let deltaTransmitted = newTransmitted - previousTransmitted;
      if (deltaReceived < 0) deltaReceived = 0;
      if (deltaTransmitted < 0) deltaTransmitted = 0;
      const throughputReceived = (deltaReceived / 1) / 1024;
      const throughputTransmitted = (deltaTransmitted / 1) / 1024;
      updateNetworkChart(
        networkChart,
        startTime,
        maxDataPoints,
        throughputReceived.toFixed(2),
        throughputTransmitted.toFixed(2)
      );
    }
    previousReceived = newReceived;
    previousTransmitted = newTransmitted;
  }
  
  // ----- Callback: Update Additional Dashboard Elements -----
  function updateAdditionalElements(data) {
    // Uptime and Load Average
    const uptimeElem = document.getElementById('uptime');
    if (uptimeElem) {
      uptimeElem.innerText = data.polled.uptime;
    }
    const loadAverageElem = document.getElementById('loadAverage');
    if (loadAverageElem) {
      loadAverageElem.innerText = `${data.polled.load_average.one} (1m), ${data.polled.load_average.five} (5m), ${data.polled.load_average.fifteen} (15m)`;
    }
    // System Details Table
    const systemDetails = document.getElementById('systemDetails');
    if (systemDetails) {
      const bootDate = new Date(data.cached.boot_time * 1000).toLocaleString();
      const totalRam = data.cached.total_ram;
      const usedRam = data.polled.used_ram;
      const totalRamGB = (totalRam / 1024 / 1024).toFixed(2);
      const usedRamGB = (usedRam / 1024 / 1024).toFixed(2);
      const totalDisk = data.cached.disk_capacity;
      const usedDisk = totalDisk > 0 ? (totalDisk - data.polled.disk_free) : 0;
      const totalDiskGB = (totalDisk / (1024 * 1024 * 1024)).toFixed(2);
      const usedDiskGB = (usedDisk / (1024 * 1024 * 1024)).toFixed(2);
      const detailsHTML = `<table>
          <tr><th>Property</th><th>Value</th></tr>
          <tr><td>🌟 OS Version</td><td>${data.cached.os_version}</td></tr>
          <tr><td>⚙️ Apache Version</td><td>${data.cached.apache_version}</td></tr>
          <tr><td>🐘 PHP Version</td><td>${data.cached.php_version}</td></tr>
          <tr><td>💽 MariaDB Version</td><td>${data.cached.mariadb_version}</td></tr>
          <tr><td>🦀 Rust Version</td><td>${data.cached.rust_version}</td></tr>
          <tr><td>📟 Node Version</td><td>${data.cached.node_version}</td></tr>
          <tr><td>📦 npm Version</td><td>${data.cached.npm_version}</td></tr>
          <tr><td>🚀 Pylon Version</td><td>${data.version}</td></tr>
          <tr><td>🔧 Processor</td><td>${data.cached.processor}</td></tr>
          <tr><td>💾 Total RAM</td><td>${totalRamGB} GB</td></tr>
          <tr><td>💿 Disk Capacity</td><td>${totalDiskGB} GB</td></tr>
          <tr><td>📀 Disk Usage</td><td>${usedDiskGB} GB</td></tr>
          <tr><td>⏰ Boot Time</td><td>${bootDate}</td></tr>
          </table>`;
      systemDetails.innerHTML = detailsHTML;
    }
    // Top Processes Table
    const topProcessesTableBody = document.getElementById('topProcessesTable')
      ? document.getElementById('topProcessesTable').getElementsByTagName('tbody')[0]
      : null;
    if (topProcessesTableBody) {
      topProcessesTableBody.innerHTML = "";
      if (data.polled.top_processes && data.polled.top_processes.length > 0) {
        data.polled.top_processes.forEach(proc => {
          const row = document.createElement('tr');
          const pidCell = document.createElement('td');
          pidCell.innerText = proc.pid;
          const nameCell = document.createElement('td');
          nameCell.innerText = proc.name;
          const memCell = document.createElement('td');
          const memMB = (proc.memory / 1024).toFixed(2);
          memCell.innerText = memMB;
          row.appendChild(pidCell);
          row.appendChild(nameCell);
          row.appendChild(memCell);
          topProcessesTableBody.appendChild(row);
        });
      }
    }
    // Services Status Lights
    const servicesDiv = document.getElementById('servicesStatus');
    if (servicesDiv) {
      servicesDiv.innerHTML = "";
      data.polled.services.forEach(service => {
        const serviceDiv = document.createElement('div');
        serviceDiv.className = "service";
        const light = document.createElement('div');
        light.className = "service-light";
        light.style.backgroundColor = service.running ? "limegreen" : "red";
        light.style.boxShadow = service.running ? "0 0 10px limegreen" : "0 0 10px red";
        const label = document.createElement('div');
        label.innerText = service.name;
        serviceDiv.appendChild(light);
        serviceDiv.appendChild(label);
        servicesDiv.appendChild(serviceDiv);
      });
    }
  }
  
  // ----- Callback: Update Remote Gauges -----
  function updateRemoteGaugesCallback(remotes) {
    window.remoteGauges = window.remoteGauges || {};
    remotes.forEach(remote => {
      const safeKey = remote.ip.replace(/\./g, '_') + "_" + remote.port;
      let remoteBlock = document.getElementById('remote_' + safeKey);
      if (!remoteBlock) {
        // Create the remote block and its gauge containers.
        remoteBlock = document.createElement('div');
        remoteBlock.id = 'remote_' + safeKey;
        remoteBlock.className = "card";
        remoteBlock.style.marginBottom = "10px";
        remoteBlock.setAttribute("data-description", remote.description || "Sorry, no description was provided for this Pylon.");
        remoteBlock.style.cursor = "pointer";
        remoteBlock.addEventListener("click", function() {
          showModal(this.getAttribute("data-description"));
        });
  
        const header = document.createElement('h3');
        header.style.margin = '0 0 10px';
        const remoteDisplayName = (remote.data && remote.data.name) ? remote.data.name : (remote.ip + ':' + remote.port);
        const remoteVersion = (remote.data && remote.data.version) ? remote.data.version : "unknown";
        const remoteLocation = remote.location || "Unknown Location";
        header.innerHTML = `🌍 <span class="pylon-server-name">${remoteDisplayName}</span>
            <span class="pylon-location">(${remoteLocation})</span>
            <span class="pylon-version">(v${remoteVersion})</span>`;
        remoteBlock.appendChild(header);
  
        const gaugesContainer = document.createElement('div');
        gaugesContainer.id = 'gauges_' + safeKey;
        gaugesContainer.style.display = "flex";
        gaugesContainer.style.flexDirection = "row";
        gaugesContainer.style.justifyContent = "space-around";
        gaugesContainer.style.overflowX = "auto";
        gaugesContainer.style.gap = "10px";
        remoteBlock.appendChild(gaugesContainer);
  
        // CPU Gauge Container
        const cpuGaugeContainer = document.createElement('div');
        cpuGaugeContainer.className = "gauge-container";
        const cpuGaugeDiv = document.createElement('div');
        const cpuGaugeId = 'cpuGauge_' + safeKey;
        cpuGaugeDiv.id = cpuGaugeId;
        cpuGaugeDiv.className = "gauge";
        cpuGaugeContainer.appendChild(cpuGaugeDiv);
        const cpuLabel = document.createElement('div');
        cpuLabel.className = "gauge-label";
        cpuLabel.innerText = "⚡ CPU";
        cpuGaugeContainer.appendChild(cpuLabel);
        gaugesContainer.appendChild(cpuGaugeContainer);
  
        // RAM Gauge Container
        const ramGaugeContainer = document.createElement('div');
        ramGaugeContainer.className = "gauge-container";
        const ramGaugeDiv = document.createElement('div');
        const ramGaugeId = 'ramGauge_' + safeKey;
        ramGaugeDiv.id = ramGaugeId;
        ramGaugeDiv.className = "gauge";
        ramGaugeContainer.appendChild(ramGaugeDiv);
        const ramLabel = document.createElement('div');
        ramLabel.className = "gauge-label";
        ramLabel.innerText = "📊 RAM";
        ramGaugeContainer.appendChild(ramLabel);
        const ramText = document.createElement('div');
        ramText.id = 'ramText_' + safeKey;
        ramText.style.fontSize = "1rem";
        ramGaugeContainer.appendChild(ramText);
        gaugesContainer.appendChild(ramGaugeContainer);
  
        // Disk Gauge Container
        const diskGaugeContainer = document.createElement('div');
        diskGaugeContainer.className = "gauge-container";
        const diskGaugeDiv = document.createElement('div');
        const diskGaugeId = 'diskGauge_' + safeKey;
        diskGaugeDiv.id = diskGaugeId;
        diskGaugeDiv.className = "gauge";
        diskGaugeContainer.appendChild(diskGaugeDiv);
        const diskLabel = document.createElement('div');
        diskLabel.className = "gauge-label";
        diskLabel.innerText = "💾 Disk";
        diskGaugeContainer.appendChild(diskLabel);
        const diskText = document.createElement('div');
        diskText.id = 'diskText_' + safeKey;
        diskText.style.fontSize = "1rem";
        diskGaugeContainer.appendChild(diskText);
        gaugesContainer.appendChild(diskGaugeContainer);
  
        // Remote Services Container
        const remoteServices = document.createElement('div');
        remoteServices.id = 'remoteServices_' + safeKey;
        remoteServices.style.display = 'flex';
        remoteServices.style.justifyContent = 'center';
        remoteServices.style.gap = '10px';
        remoteServices.style.marginTop = '10px';
        remoteBlock.appendChild(remoteServices);
  
        document.getElementById('remoteContainer').appendChild(remoteBlock);
  
        // Initialize remote gauges and store them.
        window.remoteGauges[safeKey] = {
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
              const value = (circle.value() * 100).toFixed(2);
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
              const value = (circle.value() * 100).toFixed(2);
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
              const value = (circle.value() * 100).toFixed(2);
              circle.path.setAttribute('stroke', state.color);
              circle.path.setAttribute('stroke-width', state.width);
              circle.setText(value + '%');
            }
          })
        };
        window.remoteGauges[safeKey].cpu.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
        window.remoteGauges[safeKey].cpu.text.style.fontSize = '1rem';
        window.remoteGauges[safeKey].ram.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
        window.remoteGauges[safeKey].ram.text.style.fontSize = '1rem';
        window.remoteGauges[safeKey].disk.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
        window.remoteGauges[safeKey].disk.text.style.fontSize = '1rem';
      }
  
      // Update remote gauges if online
      if (remote.online && remote.data) {
        const polled = remote.data.polled;
        const cached = remote.data.cached;
        window.remoteGauges[safeKey].cpu.animate(polled.cpu_usage / 100);
        const totalRam = cached.total_ram;
        const usedRam = polled.used_ram;
        const ramPercent = totalRam > 0 ? usedRam / totalRam : 0;
        window.remoteGauges[safeKey].ram.animate(ramPercent);
        const totalRamGB = (totalRam / 1024 / 1024).toFixed(2);
        const usedRamGB = (usedRam / 1024 / 1024).toFixed(2);
        const ramTextElem = document.getElementById('ramText_' + safeKey);
        if (ramTextElem) {
          ramTextElem.innerText = `${usedRamGB} GB / ${totalRamGB} GB`;
        }
        const totalDisk = cached.disk_capacity;
        const usedDisk = totalDisk > 0 ? (totalDisk - polled.disk_free) : 0;
        const diskPercent = totalDisk > 0 ? usedDisk / totalDisk : 0;
        window.remoteGauges[safeKey].disk.animate(diskPercent);
        const totalDiskGB = (totalDisk / (1024 * 1024 * 1024)).toFixed(2);
        const usedDiskGB = (usedDisk / (1024 * 1024 * 1024)).toFixed(2);
        const diskTextElem = document.getElementById('diskText_' + safeKey);
        if (diskTextElem) {
          diskTextElem.innerText = `${usedDiskGB} GB / ${totalDiskGB} GB`;
        }
        const remoteServicesDiv = document.getElementById('remoteServices_' + safeKey);
        if (remoteServicesDiv) {
          remoteServicesDiv.innerHTML = "";
          if (polled.services) {
            polled.services.forEach(service => {
              const serviceDiv = document.createElement('div');
              serviceDiv.className = "service";
              const light = document.createElement('div');
              light.className = "service-light";
              light.style.backgroundColor = service.running ? "limegreen" : "red";
              light.style.boxShadow = service.running ? "0 0 10px limegreen" : "0 0 10px red";
              const label = document.createElement('div');
              label.innerText = service.name;
              serviceDiv.appendChild(light);
              serviceDiv.appendChild(label);
              remoteServicesDiv.appendChild(serviceDiv);
            });
          }
        }
      } else {
        // If offline, show an error indicator.
        const displayName = remote.name || (remote.ip + ':' + remote.port);
        if (remoteBlock) {
          remoteBlock.innerHTML = `<div style="font-size:1.5rem; text-align:center;">
              ${displayName}<br><span class="pulse" style="color:red;">💻❌</span>
              </div>`;
        }
      }
    });
  }
  
  // ----- Periodic Dashboard Update -----
  async function updateDashboard() {
    await fetchLocalMetrics(updateGaugesCallback, updateAdditionalElements);
    await fetchRemoteMetrics(updateRemoteGaugesCallback);
  }
  
  setInterval(updateDashboard, 1000);
  updateDashboard();
});
