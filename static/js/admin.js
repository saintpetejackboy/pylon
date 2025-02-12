// /static/js/admin.js

/**
 * Attempts to log in the admin by sending the token to the server.
 * On success, hides the login panel and loads the admin content.
 */
export async function loginAdmin() {
  const tokenInput = document.getElementById('adminKeyInput').value;
  try {
    const response = await fetch('/api/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: tokenInput }),
    });
    if (!response.ok) {
      throw new Error("Invalid credentials");
    }
    sessionStorage.setItem('adminUnlocked', 'true');
    hideAdminLogin();
    // Load admin content (session cookie is automatically sent)
    loadAdminContent();
  } catch (err) {
    document.getElementById('adminError').style.display = 'block';
  }
}

/**
 * Hides the admin login panel.
 */
export function hideAdminLogin() {
  const adminLoginCard = document.getElementById('adminLoginCard');
  if (adminLoginCard) {
    adminLoginCard.style.display = 'none';
  }
}

/**
 * Loads admin content from the server.
 */
export async function loadAdminContent() {
  try {
    const response = await fetch('/api/admin-content');
    if (!response.ok) {
      throw new Error("Unauthorized");
    }
    const html = await response.text();
    document.getElementById('adminContent').innerHTML = html;
    // Initialize admin–only functionality (e.g. form events)
    initAdminContent();
  } catch (err) {
    console.error("Error loading admin content:", err);
    document.getElementById('adminError').style.display = 'block';
  }
}

/**
 * Initializes the admin content once it has been loaded.
 * For example, attach event listeners for managing remote pylons.
 */
export function initAdminContent() {
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
  fetchPylonConfig();
}

/**
 * Fetches the current remote pylon configuration from the server and displays it.
 */
async function fetchPylonConfig() {
  try {
    const response = await fetch('/api/config/pylons');
    const pylons = await response.json();
    const list = document.getElementById('pylonList');
    if (list) {
      list.innerHTML = "";
      if (Array.isArray(pylons)) {
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
