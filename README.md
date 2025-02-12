# Pylon Dashboard 🚀✨

<!-- Build and License -->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

<!-- Repo Metrics -->
[![Repo Size](https://img.shields.io/github/repo-size/saintpetejackboy/pylon)](https://github.com/saintpetejackboy/pylon)
[![Issues](https://img.shields.io/github/issues/saintpetejackboy/pylon)](https://github.com/saintpetejackboy/pylon/issues)
[![Top Language](https://img.shields.io/github/languages/top/saintpetejackboy/pylon)](https://github.com/saintpetejackboy/pylon)

<!-- Language Breakdown (example percentages; update as needed) -->
[![Rust](https://img.shields.io/badge/Rust-35%25-orange)](#)
[![HTML](https://img.shields.io/badge/HTML-15%25-blue)](#)
[![CSS](https://img.shields.io/badge/CSS-4.7%25-blue)](#)

<!-- Additional Cool Badges -->
[![GitHub last commit](https://img.shields.io/github/last-commit/saintpetejackboy/pylon)](https://github.com/saintpetejackboy/pylon/commits)
[![Contributors](https://img.shields.io/github/contributors/saintpetejackboy/pylon)](https://github.com/saintpetejackboy/pylon/graphs/contributors)
[![GitHub forks](https://img.shields.io/github/forks/saintpetejackboy/pylon?style=social)](https://github.com/saintpetejackboy/pylon/network)
[![GitHub stars](https://img.shields.io/github/stars/saintpetejackboy/pylon?style=social)](https://github.com/saintpetejackboy/pylon/stargazers)

---

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Installation](#installation)
- [Configuration](#configuration)
- [Building & Running](#building--running)
- [API Endpoints](#api-endpoints)
- [Contributing](#contributing)
- [Security Considerations](#security-considerations)
- [Future Roadmap](#future-roadmap)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## Features ✨

- **Real-Time System Metrics:**  
  Monitor CPU usage, RAM, disk I/O, network throughput, load average, and even view top memory-consuming processes in real time.

- **Remote Peer Discovery & Monitoring:**  
  Automatically detect and poll remote Pylon instances. The dashboard aggregates data from multiple systems so you can monitor your entire network from a single interface.

- **Dynamic Web Dashboard:**  
  A responsive HTML/JS/CSS interface featuring animated gauges, charts, and detailed panels for both local and remote systems.

- **Admin Interface:**  
  Secure admin endpoints for advanced system insights and remote pylon management (e.g., adding or removing remote configurations).

- **Hot-Reload Configuration:**  
  The application watches for changes to its TOML configuration file (`config.toml`) and reloads settings on the fly without needing a restart.

- **Graceful Shutdown:**  
  Utilizes asynchronous tasks and proper shutdown channels to ensure a clean exit, preserving data and system stability.

- **Static Binary Compilation:**  
  Option to build a fully static binary using MUSL, making deployment on Linux environments easier and more portable.

---

## Architecture

Pylon Dashboard is organized into several key modules:

### 1. Configuration Management (`config_manager.rs`)
- **Responsibilities:**
  - **Load/Save Config:** Reads settings from `config.toml` and writes changes back.
  - **Hot-Reload:** Uses file watchers to detect changes and reload the configuration dynamically.
- **Configuration Structure:**
  - **Local Settings:** Includes the local port, admin token, name, description, and location.
  - **Remote Pylons:** A list of remote instance configurations (each with IP, port, token, and optional name, location, and description).

### 2. System Information (`system_info.rs`)
- **Responsibilities:**
  - **Local Metrics:** Polls for CPU usage, RAM usage, disk stats, network activity, uptime, and system load.
  - **Static (Cached) Info:** Retrieves and caches system details like OS version, processor info, and versions of key software (e.g., Apache, PHP, MariaDB, Rust, Node.js).

### 3. Remote Monitoring (`remote.rs`)
- **Responsibilities:**
  - **Polling Remote Pylons:** Regularly fetches metrics from configured and discovered remote pylons.
  - **Peer Discovery:** Scans remote responses for additional peer configurations and integrates them into the polling list.

### 4. Web Server & API (`server.rs`)
- **Responsibilities:**
  - **Serving the UI:** Delivers the main HTML dashboard along with static assets (JavaScript, CSS, icons).
  - **RESTful Endpoints:** Provides APIs for system metrics, remote status, admin login, and remote pylon configuration management.
  - **Session Management:** Implements secure sessions (with cookie storage) for admin authentication.

### 5. Application Orchestration (`main.rs`)
- **Responsibilities:**
  - **Task Coordination:** Spawns background tasks for configuration watching, system metric polling, and remote pylon monitoring.
  - **Graceful Shutdown:** Listens for termination signals (e.g., `ctrl+c`) and shuts down all tasks cleanly.

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (Edition 2021 or later)
- [Cargo](https://doc.rust-lang.org/cargo/)
- A Linux, macOS, or Windows system for development (production deployments are typically on Linux)
- *(Optional)* [GitHub CLI (`gh`)](https://cli.github.com/) for repository management

### Clone the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/saintpetejackboy/pylon.git
cd pylon
```

---

## Configuration

Pylon Dashboard uses a TOML configuration file to manage its settings. On the first run, the application checks for a `config.toml` file in the project directory. If not found, it generates one from the embedded default configuration (`config_default.toml`).

### Configuration Options

- **Local Settings:**
  - `local_port` *(Optional)*: The default port for the web server (default is `6989`).
  - `token`: A secret token used for admin access and remote API authentication.
  - `name` *(Optional)*: The display name of this local Pylon instance.
  - `description` *(Optional)*: A short description of the local Pylon.
  - `location` *(Optional)*: The physical or logical location (e.g., "Data Center A").

- **Remote Pylons:**
  - `remote_pylons`: An array of remote configuration objects. Each remote pylon can have:
    - `ip`: IP address of the remote Pylon.
    - `port`: Port number where the remote Pylon is accessible.
    - `token`: Authentication token for connecting to the remote pylon.
    - `name` *(Optional)*: A friendly name.
    - `location` *(Optional)*: Location information (e.g., "Branch Office").
    - `description` *(Optional)*: A description of the remote pylon.

### Example `config.toml`

```toml
local_port = 6989
token = "your_secret_token"
name = "Local Pylon"
description = "Monitoring system for the primary server."
location = "Data Center A"

[[remote_pylons]]
ip = "192.168.1.10"
port = 6989
token = "remote_token_1"
name = "Remote Pylon 1"
location = "Branch Office"
description = "Backup server monitoring."
```

> **Security Note:** Ensure that `config.toml` is added to your `.gitignore` so sensitive information is not accidentally committed.

---

## Building & Running

Pylon Dashboard is built with Rust and leverages asynchronous programming via Tokio. Follow these steps to compile and run the application.

### Building a Static Binary with MUSL

For a portable, static Linux binary:

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

The compiled binary will be located at:

```
target/x86_64-unknown-linux-musl/release/pylon
```

### Running the Application

Simply execute the compiled binary:

```bash
./target/x86_64-unknown-linux-musl/release/pylon
```

On startup, the application will:
- Ensure a `config.toml` exists (or create one from the default).
- Start background tasks for system metrics polling, remote monitoring, and configuration file watching.
- Launch the web server on the default (or next available) port—by default starting at `6989`.

Open your browser and navigate to:

```
http://127.0.0.1:<port>
```

Replace `<port>` with the actual port number printed in the console.

---

## API Endpoints

Pylon Dashboard exposes several RESTful endpoints for both public and admin use:

### Public Endpoints

- **GET /**  
  Serves the main web dashboard interface. Simply open this URL in your browser.

- **GET /api/metrics**  
  Returns local system metrics (CPU, RAM, disk usage, network stats, etc.) along with cached system information.
  
  **Example Response:**
  ```json
  {
    "name": "Local Pylon",
    "description": "Monitoring system for the primary server.",
    "location": "Data Center A",
    "version": "0.2.1",
    "cached": { /* Cached system info */ },
    "polled": { /* Polled metrics */ },
    "remote_pylons": [ /* List of remote pylons from config */ ]
  }
  ```

- **GET /api/remotes**  
  Returns the current status of all remote pylons being monitored.

  **Example Response:**
  ```json
  [
    {
      "ip": "192.168.1.10",
      "port": 6989,
      "last_seen": "2025-02-11T12:34:56Z",
      "data": { /* Remote metrics */ },
      "online": true,
      "name": "Remote Pylon 1",
      "location": "Branch Office",
      "description": "Backup server monitoring."
    }
  ]
  ```

### Admin Endpoints

These endpoints require admin authentication using the token specified in your configuration.

- **POST /api/login**  
  Authenticates the admin user. On success, a session is created.

  **Payload:**
  ```json
  { "token": "your_secret_token" }
  ```
  
  **Success Response:**
  ```json
  { "status": "logged in" }
  ```

- **GET /api/admin-content**  
  Returns HTML content for the admin panel, which includes system details and remote management tools.  
  *(Accessible only after successful authentication.)*

- **GET /api/config/pylons**  
  Retrieves the list of remote pylons currently defined in the configuration.

- **POST /api/config/pylons/add**  
  Adds a new remote pylon to the configuration.

  **Payload:**
  ```json
  {
    "ip": "192.168.1.11",
    "port": 6989,
    "token": "remote_token_2",
    "name": "Remote Pylon 2",
    "location": "Remote Office",
    "description": "Secondary monitoring node."
  }
  ```

  **Success Response:**
  ```json
  { "status": "added" }
  ```

- **POST /api/config/pylons/remove**  
  Removes a remote pylon configuration.

  **Payload:**
  ```json
  { "ip": "192.168.1.11", "port": 6989 }
  ```

  **Success Response:**
  ```json
  { "status": "removed" }
  ```

---

## Contributing 🤝

Contributions to Pylon Dashboard are welcome! Whether you're fixing bugs, adding features, or improving documentation, please adhere to the following guidelines:

- **Code Style:**  
  Write clean, idiomatic Rust. Follow community best practices and comment your code as needed.

- **Testing:**  
  Ensure that your changes are tested. Add tests for new features or bug fixes when possible.

- **Pull Requests:**  
  Before submitting a PR, open an issue to discuss the change. Include a clear description of your changes and why they are necessary.

- **Documentation:**  
  Update documentation and in-code comments to reflect your changes.

Feel free to reach out to the maintainers if you have any questions or need guidance.

---

## Security Considerations

- **Configuration Files:**  
  Your `config.toml` may contain sensitive data (e.g., admin tokens). Make sure it is included in your `.gitignore` to prevent accidental commits.

- **Admin Token:**  
  Use a strong, unique token for admin authentication. Do not share this token publicly.

- **Session Security:**  
  In production, update the default session key in `server.rs` with a secure, randomly generated key. Consider deploying over HTTPS to protect your session data.

---

## Future Roadmap

- **Enhanced UI/UX:**  
  Further improvements to the dashboard design and mobile responsiveness. Alternative skins.

- **Extended Monitoring:**  
  Integration of additional system metrics with configurable options.

- **Advanced User Management:**  
  Implement better authentication tactics.

- **Cross-Platform Optimization:**  
  There are no plans to make this work on Windows or Mac, sorry.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Acknowledgments

- Built using the powerful Rust ecosystem (including [Tokio](https://tokio.rs/), [Actix Web](https://actix.rs/), and [sysinfo](https://crates.io/crates/sysinfo)).
- Inspired by classic system monitoring tools and modern dashboard interfaces.
- Thanks to the open-source community for ongoing support and contributions.

---

Happy monitoring—and may your pylons always be online! 🌟
