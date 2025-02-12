// -------------------------
// Modal Functions
// -------------------------
function showModal(description) {
  const modal = document.getElementById("pylonModal");
  const modalDescription = document.getElementById("modalDescription");
  modalDescription.innerText = description;
  modal.style.display = "block";
}

function hideModal() {
  const modal = document.getElementById("pylonModal");
  modal.style.display = "none";
}

// -------------------------
// Admin Content Loading & Initialization
// -------------------------
function loadAdminContent(token) {
  fetch('/api/admin-content', {
    headers: { 'X-Admin-Token': token }
  })
    .then(response => {
      if (!response.ok) {
        throw new Error("Unauthorized");
      }
      return response.text();
    })
    .then(html => {
      document.getElementById('adminContent').innerHTML = html;
      // Now that the admin HTML is in the DOM, initialize admin–only functionality.
      initAdminContent();
    })
    .catch(err => {
      console.error("Error loading admin content:", err);
      const adminError = document.getElementById('adminError');
      if (adminError) {
        adminError.style.display = 'block';
      }
    });
}

function initAdminContent() {
  // Attach event listener to the admin form (for managing remote pylons)
  const pylonForm = document.getElementById('pylonForm');
  if (pylonForm) {
    pylonForm.addEventListener('submit', async function(e) {
      e.preventDefault();
      const ip = document.getElementById('pylonIp').value;
      const port = parseInt(document.getElementById('pylonPort').value, 10);
      const token = document.getElementById('pylonToken').value;
      const name = document.getElementById('pylonName').value;
      const newPylon = { ip, port, token, name: name || null };

      try {
        await fetch('/api/config/pylons/add', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(newPylon)
        });
        pylonForm.reset();
        fetchPylonConfig();
      } catch (err) {
        console.error("Error adding pylon:", err);
      }
    });
  }
  // Fetch and display the current remote pylons
  fetchPylonConfig();
}

async function fetchPylonConfig() {
  try {
    const response = await fetch('/api/config/pylons');
    const pylons = await response.json();
    const list = document.getElementById('pylonList');
    if (list) {
      list.innerHTML = "";
      if (pylons && Array.isArray(pylons)) {
        pylons.forEach(pylon => {
          const li = document.createElement('li');
          li.innerText = `${pylon.ip}:${pylon.port} (${pylon.name || "No Name"})`;
          const removeBtn = document.createElement('button');
          removeBtn.innerText = "Remove";
          removeBtn.onclick = async function() {
            try {
              await fetch('/api/config/pylons/remove', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ ip: pylon.ip, port: pylon.port })
              });
              fetchPylonConfig();
            } catch (err) {
              console.error("Error removing pylon:", err);
            }
          };
          li.appendChild(removeBtn);
          list.appendChild(li);
        });
      }
    }
  } catch (err) {
    console.error("Error fetching pylon config:", err);
  }
}

// -------------------------
// Main Code (runs on DOMContentLoaded)
// -------------------------
document.addEventListener('DOMContentLoaded', function() {
  // -------------------------
  // Admin Login Functionality
  // -------------------------
  const adminKeySubmit = document.getElementById('adminKeySubmit');
  if (adminKeySubmit) {
    adminKeySubmit.addEventListener('click', function() {
      const input = document.getElementById('adminKeyInput').value;
      // A client–side check against the default token
      if (input === DEFAULT_TOKEN) {
        sessionStorage.setItem('adminUnlocked', 'true');
        loadAdminContent(input);
        const adminLoginCard = document.getElementById('adminLoginCard');
        if (adminLoginCard) {
          adminLoginCard.style.display = 'none';
        }
      } else {
        const adminError = document.getElementById('adminError');
        if (adminError) {
          adminError.style.display = 'block';
        }
      }
    });
  }
  // Auto–load admin content if already unlocked
  if (sessionStorage.getItem('adminUnlocked') === 'true') {
    loadAdminContent(DEFAULT_TOKEN);
    const adminLoginCard = document.getElementById('adminLoginCard');
    if (adminLoginCard) {
      adminLoginCard.style.display = 'none';
    }
  }

  // -------------------------
  // Modal Close Events
  // -------------------------
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

  // -------------------------
  // Local Metrics Card Click (to show description in modal)
  // -------------------------
  const localCard = document.getElementById("localMetricsCard");
  if (localCard) {
    localCard.classList.add("pylon-card");
    localCard.addEventListener("click", function() {
      const desc = localCard.getAttribute("data-description") || "Sorry, no description was provided for this Pylon.";
      showModal(desc);
    });
  }

  // -------------------------
  // Local Gauges & Charts Initialization
  // -------------------------
  // Global variable for remote gauges (used later)
  const remoteGauges = {};

  // CPU Gauge
  const cpuGauge = new ProgressBar.Circle('#cpuGauge', {
    color: '#00ff00',
    strokeWidth: 6,
    trailWidth: 2,
    easing: 'easeInOut',
    duration: 800,
    text: { value: '0.00%' },
    from: { color: '#ff4444', width: 2 },
    to: { color: '#00ff00', width: 6 },
    step: function(state, circle) {
      const value = (circle.value() * 100).toFixed(2);
      circle.path.setAttribute('stroke', state.color);
      circle.path.setAttribute('stroke-width', state.width);
      circle.setText(value + '%');
    }
  });
  cpuGauge.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
  cpuGauge.text.style.fontSize = '1.5rem';

  // RAM Gauge
  const ramGauge = new ProgressBar.Circle('#ramGauge', {
    color: '#2196F3',
    strokeWidth: 6,
    trailWidth: 2,
    easing: 'easeInOut',
    duration: 800,
    text: { value: '0.00%' },
    from: { color: '#ff5722', width: 2 },
    to: { color: '#2196F3', width: 6 },
    step: function(state, circle) {
      const value = (circle.value() * 100).toFixed(2);
      circle.path.setAttribute('stroke', state.color);
      circle.path.setAttribute('stroke-width', state.width);
      circle.setText(value + '%');
    }
  });
  ramGauge.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
  ramGauge.text.style.fontSize = '1.5rem';

  // Disk Gauge
  const diskGauge = new ProgressBar.Circle('#diskGauge', {
    color: '#ffcc00',
    strokeWidth: 6,
    trailWidth: 2,
    easing: 'easeInOut',
    duration: 800,
    text: { value: '0.00%' },
    from: { color: '#ff4444', width: 2 },
    to: { color: '#ffcc00', width: 6 },
    step: function(state, circle) {
      const value = (circle.value() * 100).toFixed(2);
      circle.path.setAttribute('stroke', state.color);
      circle.path.setAttribute('stroke-width', state.width);
      circle.setText(value + '%');
    }
  });
  diskGauge.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
  diskGauge.text.style.fontSize = '1.5rem';

  // -------------------------
  // Network Throughput Chart Setup (using Chart.js)
  // -------------------------
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

  let previousReceived = null;
  let previousTransmitted = null;
  const startTime = Date.now();
  const maxDataPoints = 30;

  function updateNetworkChart(receivedKBs, transmittedKBs) {
    const nowSec = Math.floor((Date.now() - startTime) / 1000);
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

  // -------------------------
  // Fetch Local Metrics & Update Dashboard
  // -------------------------
  async function fetchLocalMetrics() {
    try {
      const response = await fetch('/api/metrics');
      const data = await response.json();

      // Update local Pylon header
      const localPylonName = document.getElementById('localPylonName');
      if (localPylonName) {
        localPylonName.innerHTML = `<span class="pylon-server-name">${data.name}</span>
          <span class="pylon-location">(${data.location})</span> 🚀✨ 
          <span class="pylon-version">(v${data.version})</span>`;
      }

      // Update CPU gauge
      cpuGauge.animate(data.polled.cpu_usage / 100);

      // Update RAM gauge
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

      // Update Disk gauge
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

      // Update description on the local metrics card
      const localMetricsCard = document.getElementById('localMetricsCard');
      if (localMetricsCard) {
        localMetricsCard.setAttribute("data-description", data.description || "Sorry, no description was provided for this Pylon.");
      }

      // Network throughput: compute delta and update chart
      const newReceived = data.polled.network_received;
      const newTransmitted = data.polled.network_transmitted;
      if (previousReceived !== null && previousTransmitted !== null) {
        let deltaReceived = newReceived - previousReceived;
        let deltaTransmitted = newTransmitted - previousTransmitted;
        if (deltaReceived < 0) deltaReceived = 0;
        if (deltaTransmitted < 0) deltaTransmitted = 0;
        const throughputReceived = (deltaReceived / 1) / 1024;
        const throughputTransmitted = (deltaTransmitted / 1) / 1024;
        updateNetworkChart(throughputReceived.toFixed(2), throughputTransmitted.toFixed(2));
      }
      previousReceived = newReceived;
      previousTransmitted = newTransmitted;

      // If admin content is loaded, update additional elements:
      const uptimeElem = document.getElementById('uptime');
      if (uptimeElem) {
        uptimeElem.innerText = data.polled.uptime;
      }
      const loadAverageElem = document.getElementById('loadAverage');
      if (loadAverageElem) {
        loadAverageElem.innerText = `${data.polled.load_average.one} (1m), ${data.polled.load_average.five} (5m), ${data.polled.load_average.fifteen} (15m)`;
      }

      const systemDetails = document.getElementById('systemDetails');
      if (systemDetails) {
        const bootDate = new Date(data.cached.boot_time * 1000).toLocaleString();
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
    } catch (err) {
      console.error('Error fetching local metrics:', err);
    }
  }

  async function fetchRemoteMetrics() {
    try {
      // Optional remote poll indicator
      const indicator = document.getElementById('remotePollIndicator');
      if (indicator) {
        indicator.innerHTML = '<span class="pulse">🌐</span>';
        setTimeout(() => { indicator.innerHTML = ''; }, 1500);
      }

      const response = await fetch('/api/remotes');
      const remotes = await response.json();
      const container = document.getElementById('remoteContainer');

      remotes.forEach(remote => {
        const safeKey = remote.ip.replace(/\./g, '_') + "_" + remote.port;
        let remoteBlock = document.getElementById('remote_' + safeKey);
        if (!remoteBlock) {
          remoteBlock = document.createElement('div');
          remoteBlock.id = 'remote_' + safeKey;
          remoteBlock.className = "card";
          remoteBlock.style.marginBottom = "10px";
          remoteBlock.setAttribute("data-description", remote.description ? remote.description : "Sorry, no description was provided for this Pylon.");
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

          // CPU Gauge for remote
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

          // RAM Gauge for remote
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

          // Disk Gauge for remote
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

          // Remote Services Status Lights Container
          const remoteServices = document.createElement('div');
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
          remoteGauges[safeKey].cpu.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
          remoteGauges[safeKey].cpu.text.style.fontSize = '1rem';
          remoteGauges[safeKey].ram.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
          remoteGauges[safeKey].ram.text.style.fontSize = '1rem';
          remoteGauges[safeKey].disk.text.style.fontFamily = '"Segoe UI", Tahoma, Geneva, Verdana, sans-serif';
          remoteGauges[safeKey].disk.text.style.fontSize = '1rem';
        }

        // Update remote gauges if the remote is online
        if (remote.online && remote.data) {
          const polled = remote.data.polled;
          const cached = remote.data.cached;
          remoteGauges[safeKey].cpu.animate(polled.cpu_usage / 100);

          const totalRam = cached.total_ram;
          const usedRam = polled.used_ram;
          const ramPercent = totalRam > 0 ? usedRam / totalRam : 0;
          remoteGauges[safeKey].ram.animate(ramPercent);
          const totalRamGB = (totalRam / 1024 / 1024).toFixed(2);
          const usedRamGB = (usedRam / 1024 / 1024).toFixed(2);
          const ramTextElem = document.getElementById('ramText_' + safeKey);
          if (ramTextElem) {
            ramTextElem.innerText = `${usedRamGB} GB / ${totalRamGB} GB`;
          }

          const totalDisk = cached.disk_capacity;
          const usedDisk = totalDisk > 0 ? (totalDisk - polled.disk_free) : 0;
          const diskPercent = totalDisk > 0 ? usedDisk / totalDisk : 0;
          remoteGauges[safeKey].disk.animate(diskPercent);
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
          const displayName = remote.name ? remote.name : (remote.ip + ':' + remote.port);
          const remoteBlock = document.getElementById('remote_' + safeKey);
          if (remoteBlock) {
            remoteBlock.innerHTML = `<div style="font-size:1.5rem; text-align:center;">
              ${displayName}<br><span class="pulse" style="color:red;">💻❌</span>
              </div>`;
          }
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

  // Start periodic updates
  setInterval(updateDashboard, 1000);
  updateDashboard();
});
