// /static/js/dashboard.js

/**
 * Initializes local gauges and the network chart.
 * Returns an object containing the created gauges and chart.
 */
export function initGauges() {
  // Initialize the CPU gauge
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

  // Initialize the RAM gauge
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

  // Initialize the Disk gauge
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

  // Initialize the Network Throughput Chart using Chart.js
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

  return { cpuGauge, ramGauge, diskGauge, networkChart };
}

/**
 * Updates the network chart.
 */
export function updateNetworkChart(networkChart, startTime, maxDataPoints, throughputReceived, throughputTransmitted) {
  const nowSec = Math.floor((Date.now() - startTime) / 1000);
  networkChart.data.labels.push(nowSec);
  networkChart.data.datasets[0].data.push(throughputReceived);
  networkChart.data.datasets[1].data.push(throughputTransmitted);
  if (networkChart.data.labels.length > maxDataPoints) {
    networkChart.data.labels.shift();
    networkChart.data.datasets[0].data.shift();
    networkChart.data.datasets[1].data.shift();
  }
  networkChart.update();
}

/**
 * Fetches local metrics from the server and calls the provided callbacks
 * to update gauges and additional dashboard elements.
 */
export async function fetchLocalMetrics(updateGauges, updateAdditionalElements) {
  try {
    const response = await fetch('/api/metrics');
    const data = await response.json();

    // Update the header text
    const localPylonName = document.getElementById('localPylonName');
    if (localPylonName) {
      localPylonName.innerHTML = `<span class="pylon-server-name">${data.name}</span>
          <span class="pylon-location">(${data.location})</span> 🚀✨ 
          <span class="pylon-version">(v${data.version})</span>`;
    }
    // Update gauges and extra elements using the provided callbacks.
    updateGauges(data);
    updateAdditionalElements(data);
  } catch (err) {
    console.error('Error fetching local metrics:', err);
  }
}

/**
 * Fetches remote metrics from the server and passes them to the callback.
 */
export async function fetchRemoteMetrics(updateRemoteGauges) {
  try {
    const indicator = document.getElementById('remotePollIndicator');
    if (indicator) {
      indicator.innerHTML = '<span class="pulse">🌐</span>';
      setTimeout(() => { indicator.innerHTML = ''; }, 1500);
    }
    const response = await fetch('/api/remotes');
    const remotes = await response.json();
    updateRemoteGauges(remotes);
  } catch (err) {
    console.error('Error fetching remote metrics:', err);
  }
}
