# h8r: HAProxy Resource Manager

h8r is a command-line interface (CLI) tool designed for interacting with HAProxy instances in a manner similar to htop and k9s. It provides a streamlined and efficient way to monitor and manage HAProxy resources directly from the terminal.

## Features

- **Interactive Interface:** h8r offers an intuitive and interactive command-line interface for monitoring HAProxy instances.
- **Real-time Metrics:** View real-time metrics such as server status, backend health, and traffic statistics at a glance.
- **Resource Navigation:** Easily navigate through HAProxy resources including backends, frontends, servers, and ACLs.
- **Search and Filter:** Quickly search and filter through HAProxy resources to find specific information.
- **Customization:** Tailor h8r to your needs with customizable configuration options.

## Installation

### Prerequisites

Before installing h8r, ensure you have Rust and Cargo installed. You can install Rust and Cargo using [rustup](https://rustup.rs/).

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/h8r.git

# Navigate into the h8r directory
cd h8r

# Build the project
cargo build --release

# Run h8r
./target/release/h8r
```

### Via Cargo

```bash
# Install h8r from crates.io
cargo install h8r

# Run h8r
h8r
```

## Usage

```bash
# Run h8r
./h8r

# Alternatively, if you've installed h8r via Cargo
h8r
```

### Keyboard Shortcuts

- **Arrow keys:** Navigate through the interface
- **Enter:** Select a resource for detailed information
- **Esc or q:** Exit h8r

## Configuration

h8r can be configured to suit your preferences. The configuration file can be found at `~/.h8r/config.yaml`. Edit this file to customize settings such as colors, keybindings, and default views.

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvements, please feel free to open an issue or submit a pull request on [GitHub](https://github.com/yourusername/h8r).

## License

This project is licensed under the [MIT License](LICENSE).
