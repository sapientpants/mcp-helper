# MCP Helper

A CLI tool for MCP (Model Context Protocol) operations with subcommands, colored output, and error handling.

## Features

- **Hello Command** - Greet someone with customizable repetition
- **Calculate Command** - Perform basic arithmetic operations (add, subtract, multiply, divide)
- **Info Command** - Display application information
- **Colored Output** - Beautiful terminal output with colors
- **Verbose Mode** - Optional detailed output

## Installation

1. Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/)

2. Clone this repository and navigate to the project directory:
```bash
cd mcp-helper
```

3. Build the project:
```bash
cargo build --release
```

The binary will be available at `target/release/mcp-helper`

## Usage

### Basic Usage

```bash
# Show help
cargo run -- --help

# Or if you've built the release binary
./target/release/mcp-helper --help
```

### Commands

#### Hello Command
Greet someone with an optional repetition count:

```bash
# Default greeting
cargo run -- hello

# Greet a specific person
cargo run -- hello --name Alice

# Greet multiple times
cargo run -- hello --name Bob --count 3
```

#### Calculate Command
Perform arithmetic operations:

```bash
# Addition
cargo run -- calculate add 10 5

# Subtraction
cargo run -- calculate subtract 20 8

# Multiplication
cargo run -- calculate multiply 7 6

# Division
cargo run -- calculate divide 100 4
```

#### Info Command
Display application information:

```bash
# Basic info
cargo run -- info

# Detailed information
cargo run -- info --detailed
```

### Global Options

- `--verbose` or `-v`: Enable verbose output for additional information

```bash
cargo run -- --verbose calculate add 5 3
```

## Development

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatting

```bash
cargo fmt
```

## Dependencies

- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [anyhow](https://github.com/dtolnay/anyhow) - Flexible error handling
- [colored](https://github.com/colored-rs/colored) - Colored terminal output

## License

This project is open source and available under the MIT License.