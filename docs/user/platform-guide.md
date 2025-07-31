# MCP Helper Platform Guide

This guide covers platform-specific considerations, optimizations, and best practices for using MCP Helper on Windows, macOS, and Linux.

## Windows

### System Requirements

**Minimum Requirements:**
- Windows 10 version 1809 or later
- Windows 11 (all versions)
- PowerShell 5.1 or PowerShell 7+
- Windows Terminal (recommended)

**Architecture Support:**
- x64 (Intel/AMD 64-bit)
- ARM64 (Windows on ARM)

### Installation

**Using winget (Recommended):**
```powershell
# Install MCP Helper (when available)
winget install mcphelper

# Verify installation
mcp --version
```

**Manual Installation:**
1. Download `mcp-helper-windows-x64.exe` from [releases](https://github.com/sapientpants/mcp-helper/releases)
2. Place in a directory in your PATH (e.g., `C:\Program Files\mcp-helper\`)
3. Add to PATH if needed:
   ```powershell
   # Add to user PATH
   $env:PATH += ";C:\Program Files\mcp-helper"
   
   # Make permanent
   [Environment]::SetEnvironmentVariable("PATH", $env:PATH, "User")
   ```

### Platform-Specific Features

**Path Handling:**
- Automatically converts Unix-style paths (`/home/user`) to Windows paths (`C:\Users\user`)
- Supports both forward slashes and backslashes in configuration
- UNC path support for network drives (`\\server\share`)

**PowerShell Integration:**
```powershell
# MCP Helper works seamlessly with PowerShell
mcp install @modelcontextprotocol/server-filesystem `
  --config allowedDirectories="$env:USERPROFILE\Documents,$env:USERPROFILE\Projects"

# Use PowerShell variables in config
mcp install docker:postgres:13 `
  --config environment="POSTGRES_PASSWORD=$env:DB_PASSWORD"
```

**Windows-Specific Commands:**
- Uses `cmd.exe` for process execution when needed
- `npx.cmd` instead of `npx` for NPM packages
- Automatic handling of `.exe` extensions

### Configuration Locations

**Claude Desktop:**
```
%APPDATA%\Claude\claude_desktop_config.json
# Usually: C:\Users\YourName\AppData\Roaming\Claude\claude_desktop_config.json
```

**VS Code:**
```
%APPDATA%\Code\User\mcp.json
# Usually: C:\Users\YourName\AppData\Roaming\Code\User\mcp.json
```

**Cursor:**
```
%USERPROFILE%\.cursor\mcp.json
# Usually: C:\Users\YourName\.cursor\mcp.json
```

### Common Windows Issues

**Long Path Support:**
```powershell
# Enable long paths (Windows 10 1607+, requires admin)
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
                 -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

**Execution Policy:**
```powershell
# Allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Windows Defender:**
- Add MCP Helper directory to exclusions if needed
- Some antivirus may flag downloaded binaries

**File Associations:**
```powershell
# Associate .mcp files with MCP Helper (optional)
assoc .mcp=MCPHelperFile
ftype MCPHelperFile="C:\Program Files\mcp-helper\mcp.exe" run "%1"
```

### Performance Optimizations

**Faster Startup:**
- Place MCP Helper in an SSD location
- Exclude from Windows Defender real-time scanning
- Use Windows Terminal instead of Command Prompt

**Memory Usage:**
- Windows typically uses ~2-3MB more memory than Linux/macOS
- Close unnecessary applications during large server installations

### Windows-Specific Examples

**File System Server with Windows Paths:**
```powershell
mcp install @modelcontextprotocol/server-filesystem `
  --config allowedDirectories="C:\Projects,D:\Data,\\server\shared" `
  --config allowedFileTypes=".cs,.ps1,.bat,.md"
```

**Docker with Windows Containers:**
```powershell
# Ensure Docker Desktop is set to Windows containers if needed
mcp install docker:mcr.microsoft.com/windows/nanoserver:ltsc2022
```

**Environment Variables:**
```powershell
# Use Windows environment variables
mcp install some-server `
  --config workingDir="$env:USERPROFILE\workspace" `
  --config logPath="$env:TEMP\mcp-logs"
```

## macOS

### System Requirements

**Minimum Requirements:**
- macOS 10.15 Catalina or later
- macOS 11+ recommended for best compatibility
- Xcode Command Line Tools (for some servers)

**Architecture Support:**
- Intel x64 (Intel Macs)
- Apple Silicon ARM64 (M1, M2, M3+ Macs)

### Installation

**Using Homebrew (Recommended):**
```bash
# Install MCP Helper (when available)
brew install mcphelper

# Or install from cask
brew install --cask mcp-helper

# Verify installation
mcp --version
```

**Manual Installation:**
```bash
# Download appropriate binary
curl -L -o mcp https://github.com/sapientpants/mcp-helper/releases/download/v0.1.0/mcp-helper-macos-universal

# Make executable and install
chmod +x mcp
sudo mv mcp /usr/local/bin/

# Or install to user directory
mkdir -p ~/.local/bin
mv mcp ~/.local/bin/
# Add ~/.local/bin to PATH in ~/.zshrc or ~/.bash_profile
```

### Platform-Specific Features

**Universal Binary:**
- Single binary works on both Intel and Apple Silicon Macs
- Automatic architecture detection and optimization

**Rosetta 2 Compatibility:**
- Intel-only servers work on Apple Silicon via Rosetta 2
- MCP Helper detects and handles architecture mismatches

**macOS Security:**
```bash
# Remove quarantine attribute if needed
xattr -d com.apple.quarantine /usr/local/bin/mcp

# Or approve in System Preferences → Security & Privacy
```

**Keychain Integration:**
- Future versions will support storing sensitive config in Keychain
- Currently uses standard file permissions (600)

### Configuration Locations

**Claude Desktop:**
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**VS Code:**
```
~/Library/Application Support/Code/User/mcp.json
```

**Cursor:**
```
~/.cursor/mcp.json
```

### Common macOS Issues

**Gatekeeper Warnings:**
```bash
# If you get "unidentified developer" warning:
# 1. Right-click → Open (one-time bypass)
# 2. Or remove quarantine:
xattr -d com.apple.quarantine mcp

# 3. Or approve in System Preferences
```

**Homebrew Conflicts:**
```bash
# If multiple Node.js versions cause issues:
brew doctor
brew cleanup

# Use Homebrew's Node.js consistently
brew unlink node && brew link node

# Fix permissions
sudo chown -R $(whoami) $(brew --prefix)/*
```

**SIP (System Integrity Protection):**
- MCP Helper respects SIP restrictions
- Cannot modify system directories even with sudo
- Use user directories for installations

### Performance Optimizations

**Apple Silicon Optimizations:**
- Native ARM64 performance on M1/M2/M3+ Macs
- Faster startup and lower memory usage
- Better battery life during operation

**Spotlight Exclusions:**
```bash
# Exclude MCP Helper directories from Spotlight indexing (optional)
sudo mdutil -i off ~/.mcp
```

### macOS-Specific Examples

**File System Server with macOS Paths:**
```bash
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="$HOME/Documents,$HOME/Projects,/Volumes/External" \
  --config allowedFileTypes=".swift,.m,.h,.plist,.md"
```

**Docker Desktop Integration:**
```bash
# Ensure Docker Desktop is running
open -a Docker

# Install Docker-based server
mcp install docker:nginx:alpine \
  --config ports="8080:80" \
  --config volumes="$PWD:/usr/share/nginx/html"
```

**Python with Homebrew:**
```bash
# Use Homebrew Python consistently
mcp install python-mcp-server \
  --config pythonPath="$(brew --prefix)/bin/python3"
```

## Linux

### System Requirements

**Supported Distributions:**
- Ubuntu 18.04 LTS or later
- Debian 10 or later
- CentOS 7+ / RHEL 7+
- Fedora 32+
- openSUSE Leap 15+
- Arch Linux
- Alpine Linux 3.12+

**Architecture Support:**
- x86_64 (Intel/AMD 64-bit)
- ARM64 (aarch64)
- ARMv7 (32-bit ARM, limited)

### Installation

**Ubuntu/Debian:**
```bash
# Using apt (when repository is available)
sudo apt update
sudo apt install mcp-helper

# Or install .deb package
wget https://github.com/sapientpants/mcp-helper/releases/download/v0.1.0/mcp-helper_0.1.0_amd64.deb
sudo dpkg -i mcp-helper_0.1.0_amd64.deb
```

**CentOS/RHEL/Fedora:**
```bash
# Using dnf/yum (when repository is available)
sudo dnf install mcp-helper

# Or install .rpm package
wget https://github.com/sapientpants/mcp-helper/releases/download/v0.1.0/mcp-helper-0.1.0-1.x86_64.rpm
sudo rpm -i mcp-helper-0.1.0-1.x86_64.rpm
```

**Arch Linux:**
```bash
# Using AUR helper (when available)
yay -S mcp-helper

# Or build from source
git clone https://aur.archlinux.org/mcp-helper.git
cd mcp-helper
makepkg -si
```

**Generic Linux (AppImage):**
```bash
# Download AppImage
wget https://github.com/sapientpants/mcp-helper/releases/download/v0.1.0/MCP-Helper-x86_64.AppImage
chmod +x MCP-Helper-x86_64.AppImage

# Run directly or install
./MCP-Helper-x86_64.AppImage --install
```

**From Source:**
```bash
# Requires Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

git clone https://github.com/sapientpants/mcp-helper.git
cd mcp-helper
cargo build --release
sudo cp target/release/mcp /usr/local/bin/
```

### Platform-Specific Features

**Systemd Integration:**
```bash
# Create systemd service for auto-starting servers (future feature)
sudo systemctl enable mcp-helper
sudo systemctl start mcp-helper
```

**AppArmor/SELinux Compatibility:**
- MCP Helper includes security profiles
- Works with enforcing SELinux policies
- Respects AppArmor restrictions

**Container Support:**
- Native Docker/Podman integration
- Supports rootless containers
- Works in LXC/Docker containers

### Configuration Locations

**Claude Desktop:**
```
~/.config/Claude/claude_desktop_config.json
```

**VS Code:**
```
~/.config/Code/User/mcp.json
```

**Cursor:**
```
~/.cursor/mcp.json
```

**System-wide (optional):**
```
/etc/mcp-helper/config.json
```

### Common Linux Issues

**Permission Issues:**
```bash
# Ensure user owns config directories
sudo chown -R $USER:$USER ~/.config/Claude/
chmod 755 ~/.config/Claude/
chmod 644 ~/.config/Claude/claude_desktop_config.json

# Docker group membership
sudo usermod -aG docker $USER
newgrp docker  # Or logout/login
```

**SELinux Issues:**
```bash
# Check SELinux status
getenforce

# Temporarily disable (not recommended for production)
sudo setenforce 0

# Or configure proper SELinux policies
sudo setsebool -P httpd_can_network_connect on
```

**Missing Dependencies:**
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL/Fedora
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel pkg-config
```

### Performance Optimizations

**Memory Usage:**
- Linux typically uses the least memory (~15-20MB)
- Use `mcp install --dry-run` to estimate resource needs
- Consider memory limits for Docker containers

**Disk I/O:**
```bash
# Use faster I/O scheduler for SSDs
echo mq-deadline | sudo tee /sys/block/sda/queue/scheduler

# Mount tmp with sufficient space
sudo mount -o remount,size=2G /tmp
```

**CPU Scheduling:**
```bash
# Use nice for background operations
nice -n 10 mcp install large-server

# Or use ionice for I/O intensive operations
ionice -c 3 mcp install docker:large-image
```

### Linux-Specific Examples

**File System Server with Linux Paths:**
```bash
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="/home/$USER/Projects,/var/data,/mnt/shared" \
  --config allowedFileTypes=".py,.sh,.conf,.log,.md"
```

**Docker with Rootless Mode:**
```bash
# Use rootless Docker
export DOCKER_HOST=unix:///run/user/$(id -u)/docker.sock

mcp install docker:alpine:latest \
  --config environment="USER_ID=$(id -u),GROUP_ID=$(id -g)"
```

**Python Virtual Environments:**
```bash
# Use system Python with venv
python3 -m venv ~/.mcp/venvs/myserver
source ~/.mcp/venvs/myserver/bin/activate

mcp install python-server \
  --config pythonPath="$HOME/.mcp/venvs/myserver/bin/python"
```

**Systemd Service Integration:**
```bash
# Create user service (future feature)
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/mcp-server.service << EOF
[Unit]
Description=MCP Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/mcp run server-name
Restart=always
User=%i

[Install]
WantedBy=default.target
EOF

systemctl --user enable mcp-server
systemctl --user start mcp-server
```

## Cross-Platform Considerations

### Configuration Portability

**Relative Paths:**
```bash
# Use relative paths when possible
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="./projects,../data"
```

**Environment Variables:**
```bash
# Cross-platform environment variable usage
# Windows: %USERPROFILE%, %APPDATA%
# macOS/Linux: $HOME, $USER

mcp install server-name \
  --config workingDir='$HOME/workspace'  # Works everywhere
```

**Path Separators:**
- MCP Helper automatically handles `/` vs `\` differences
- Always use forward slashes in configuration - they work everywhere
- Windows paths with spaces: use quotes consistently

### Network and Security

**Firewall Configuration:**
- Windows: Windows Defender Firewall
- macOS: Built-in firewall (`pfctl`)
- Linux: `iptables`, `ufw`, or `firewalld`

**SSL Certificate Handling:**
- Windows: Uses Windows Certificate Store
- macOS: Uses Keychain certificates
- Linux: Uses system CA bundle (`/etc/ssl/certs/`)

### Development Workflow

**Multi-Platform Testing:**
```bash
# Test configuration on all platforms
mcp install server-name --dry-run --verbose

# Use platform-agnostic paths
mcp install docker:server \
  --config volumes="$(pwd):/workspace"
```

**CI/CD Integration:**
```yaml
# GitHub Actions example
- name: Test MCP Helper
  run: |
    mcp install test-server --dry-run
    mcp --version
  env:
    CI: true
```

This platform guide ensures MCP Helper works optimally across all supported operating systems while respecting platform-specific conventions and security models.