# Pylon Dashboard 🚀

Pylon Dashboard is a cross-platform system monitoring and peer discovery tool inspired by a classic StarCraft reference. It provides real-time metrics for your local system and remote peers, featuring a sleek web dashboard and an API interface. Whether you’re monitoring CPU usage or discovering new remote “pylons” on the network, Pylon Dashboard keeps you informed with style and precision.

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Building & Running](#building--running)
- [Deploying with GitHub CLI](#deploying-with-github-cli)
- [Contributing](#contributing)
- [License](#license)

---

## Features ✨

- **Real-time System Metrics:** Monitor CPU, RAM, disk usage, network throughput, and more.
- **Remote Peer Discovery:** Automatically detect and poll remote Pylon instances.
- **Sleek Dashboard:** A responsive, modern UI with animated gauges and charts.
- **Graceful Shutdown:** Clean exit for all background tasks on user interruption.
- **Customizable Configuration:** Easily tweak your settings with a TOML config file.
- **Compiled as a Static Binary:** Build with MUSL for portability on Linux.

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)
- [Cargo](https://doc.rust-lang.org/cargo/)
- Ubuntu (or any Linux distribution)
- GitHub CLI (`gh`) – [Installation Instructions](https://cli.github.com/)

### Clone the Repository

If you haven’t already, clone the repository (once it’s live on GitHub):

```bash
git clone https://github.com/saintpetejackboy/pylon.git
cd pylon
```

---

## Configuration

### Configuration File

- **config_default.toml**: Provides the default configuration.
- **config.toml**: This is generated on first run if missing.  
  **Note:** It is safe to leave your actual `config.toml` in the project directory as long as you add it to your `.gitignore`. This prevents sensitive data (such as tokens) from being pushed to GitHub.

Your `.gitignore` already includes:

```
/target
config.toml
```

This means your local configuration will **not** be committed.

---

## Building & Running

### Build a Static Binary

To compile a release binary for Linux using MUSL, run:

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

The binary will be located at:  
`target/x86_64-unknown-linux-musl/release/pylon`

### Run the Application

After building, you can run the application:

```bash
./target/x86_64-unknown-linux-musl/release/pylon
```

The server will automatically select an available port (default is `6989`), and you can view the dashboard by navigating to `http://127.0.0.1:<port>` in your web browser.

---

## Deploying with GitHub CLI

Assuming your local project is in `/home/jack/pylon`, here’s how to create a new public repository called **pylon** on GitHub using your CLI, then push your code.

1. **Navigate to your project directory:**

   ```bash
   cd /home/jack/pylon
   ```

2. **Create the repository using `gh`:**

   ```bash
   gh repo create saintpetejackboy/pylon --public --source=. --remote=origin --push
   ```

   This command will:
   - Create a new public repository on GitHub under your username.
   - Set your local repository’s remote to `origin`.
   - Push your current code to the remote repository.

3. **If you need to initialize Git manually (if not already initialized):**

   ```bash
   git init
   git add .
   git commit -m "Initial commit of Pylon Dashboard"
   gh repo create saintpetejackboy/pylon --public --source=. --remote=origin --push
   ```

4. **Verify that everything is working:**

   Open your browser to `https://github.com/saintpetejackboy/pylon` to see your repository.

---

## Contributing 🤝

Contributions are welcome! Feel free to open issues or submit pull requests. When contributing, please follow these guidelines:

- Ensure your code is well-commented.
- Follow Rust’s idiomatic practices.
- Test your changes before submitting.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

Happy monitoring and may your pylons always be online! 🌟
```

