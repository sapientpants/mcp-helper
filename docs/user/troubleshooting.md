# MCP Helper Troubleshooting Guide

This guide helps you resolve common issues when using MCP Helper to install and manage MCP servers.

## Quick Diagnostics

### First Steps
1. **Check MCP Helper version**: `mcp --version`
2. **Run with verbose mode**: Add `--verbose` to any command
3. **Try dry-run mode**: Add `--dry-run` to see what would happen
4. **Check system requirements**: Node.js, Docker, Python (depending on server type)

### Verbose Mode Output
When you run commands with `--verbose`, you'll see detailed information:
```bash
mcp install server-name --verbose

# Example output:
# ℹ Detected platform: Linux
# ✓ Found Node.js v20.11.0  
# ✓ Security validation passed
# → Installing to Claude Desktop...
# ✓ Configuration snapshot saved: 2024-01-15 10:30:45
```

## Common Installation Issues

### 1. Node.js Problems

#### "Node.js not found" or "npx not available"

**Symptoms:**
```
✗ Node.js dependency check failed
Missing dependency: Node.js
```

**Solutions:**

**Windows:**
```bash
# Using winget (recommended)
winget install OpenJS.NodeJS

# Using Chocolatey
choco install nodejs

# Or download from: https://nodejs.org/
```

**macOS:**
```bash
# Using Homebrew (recommended)  
brew install node

# Using MacPorts
sudo port install nodejs18

# Or download from: https://nodejs.org/
```

**Linux (Ubuntu/Debian):**
```bash
# Using apt
sudo apt update
sudo apt install nodejs npm

# Using snap
sudo snap install node --classic
```

**Linux (CentOS/RHEL/Fedora):**
```bash
# CentOS/RHEL
sudo yum install nodejs npm

# Fedora
sudo dnf install nodejs npm
```

#### "npx command not found" (with Node.js installed)

**Solution:**
```bash
# Reinstall npm
npm install -g npm@latest

# Or install npx separately
npm install -g npx

# Verify both work
node --version
npx --version
```

#### Version Compatibility Issues

**Symptoms:**
```
✗ Node.js version mismatch (v16.0.0 < v18.0.0)
Server requires Node.js v18.0.0 or higher
```

**Solutions:**
- Update Node.js to the required version
- Use a Node.js version manager (nvm, fnm)

**Using nvm (macOS/Linux):**
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install and use latest LTS
nvm install --lts
nvm use --lts
nvm alias default node
```

**Using fnm (cross-platform):**
```bash
# Install fnm
curl -fsSL https://fnm.vercel.app/install | bash

# Install and use latest LTS
fnm install --lts
fnm use lts-latest
fnm default lts-latest
```

### 2. Docker Problems

#### "Docker not found" or "Docker not running"

**Symptoms:**
```
✗ Docker dependency check failed
Docker is installed but not running
```

**Solutions:**

**Windows/macOS:**
```bash
# Start Docker Desktop
# Check system tray/menu bar for Docker icon
# If not installed: download from https://desktop.docker.com/
```

**Linux:**
```bash
# Start Docker daemon
sudo systemctl start docker
sudo systemctl enable docker  # Auto-start on boot

# Add your user to docker group (logout/login required)
sudo usermod -aG docker $USER

# Test Docker
docker --version
docker ps
```

#### Docker Permission Issues (Linux)

**Symptoms:**
```
✗ Docker command failed: permission denied
```

**Solution:**
```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Apply group changes (logout/login or newgrp)
newgrp docker

# Or run with sudo (not recommended for regular use)
sudo mcp install docker:server-name
```

#### "Docker Compose not available"

**Solution:**
```bash
# Modern Docker includes Compose v2
docker compose version

# If not available, install docker-compose
pip install docker-compose
# Or use your package manager
```

### 3. Python Problems

#### "Python not found"

**Symptoms:**
```
✗ Python dependency check failed
Missing dependency: Python 3.8+
```

**Solutions:**

**Windows:**
```bash
# Using winget
winget install Python.Python.3.11

# Using Python installer
# Download from: https://python.org/downloads/
```

**macOS:**
```bash
# Using Homebrew
brew install python@3.11

# Using system Python (may be outdated)
# Install from: https://python.org/downloads/
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install python3 python3-pip python3-venv

# CentOS/RHEL/Fedora
sudo dnf install python3 python3-pip  # Fedora
sudo yum install python3 python3-pip  # CentOS/RHEL
```

#### Virtual Environment Issues

**Symptoms:**
```
✗ Failed to create Python virtual environment
```

**Solutions:**
```bash
# Install venv module
python3 -m pip install --user virtualenv

# Or use system package
sudo apt install python3-venv  # Ubuntu/Debian
```

### 4. Client Detection Issues

#### "No MCP clients found"

**Symptoms:**
```
✗ No MCP clients selected for installation
Available clients: (none found)
```

**Solutions:**

**For Claude Desktop:**
1. **Check if Claude Desktop is installed**:
   - Windows: Look in `%LOCALAPPDATA%\Programs\Claude` or Program Files
   - macOS: Look in `/Applications/Claude.app`
   - Linux: Check installation directory

2. **Check config directory exists**:
   - Windows: `%APPDATA%\Claude\`
   - macOS: `~/Library/Application Support/Claude/`
   - Linux: `~/.config/Claude/`

3. **Create config directory if missing**:
   ```bash
   # Windows (PowerShell)
   New-Item -ItemType Directory -Path "$env:APPDATA\Claude" -Force
   
   # macOS/Linux
   mkdir -p ~/.config/Claude/  # Linux
   mkdir -p ~/Library/Application\ Support/Claude/  # macOS
   ```

**For other clients:**
- **Cursor**: Check for `.cursor` directory in home folder
- **VS Code**: Verify installation and GitHub Copilot extension
- **Windsurf**: Check for `.codeium/windsurf` directory

#### Client Config Corruption

**Symptoms:**
```
✗ Failed to parse client configuration
JSON syntax error in config file
```

**Solutions:**
1. **Backup and repair config**:
   ```bash
   # Backup current config
   cp claude_desktop_config.json claude_desktop_config.json.backup
   
   # Validate JSON syntax
   python -m json.tool claude_desktop_config.json
   
   # Or use online JSON validator
   ```

2. **Reset to minimal config**:
   ```json
   {
     "mcpServers": {}
   }
   ```

3. **MCP Helper creates automatic backups** - check for `.backup` files

### 5. Network and Security Issues

#### "Failed to download server"

**Symptoms:**
```
✗ Failed to fetch package information
Network timeout or connection refused
```

**Solutions:**
1. **Check internet connection**
2. **Corporate firewall/proxy**:
   ```bash
   # Set proxy environment variables
   export HTTP_PROXY=http://proxy.company.com:8080
   export HTTPS_PROXY=http://proxy.company.com:8080
   export NO_PROXY=localhost,127.0.0.1,.local
   
   # Or configure npm proxy
   npm config set proxy http://proxy.company.com:8080
   npm config set https-proxy http://proxy.company.com:8080
   ```

3. **DNS issues**:
   ```bash
   # Test DNS resolution
   nslookup npmjs.org
   nslookup github.com
   
   # Try different DNS servers
   # Add to /etc/resolv.conf (Linux) or network settings:
   # nameserver 8.8.8.8
   # nameserver 1.1.1.1
   ```

#### Security Warnings

**Symptoms:**
```
⚠ Security warnings detected:
  • Domain 'unknown.com' is not in the list of trusted sources
  • Package name matches system command
```

**Solutions:**
1. **Review the warnings carefully**
2. **For legitimate packages**: Choose to proceed when prompted
3. **For suspicious packages**: Cancel installation and verify the source
4. **Add trusted domains** (advanced): Modify MCP Helper configuration

#### "Installation blocked due to security concerns"

**Symptoms:**
```
✗ Installation blocked due to security concerns
```

**Solutions:**
1. **Verify the package source** is legitimate
2. **Check for typos** in package names (typosquatting protection)
3. **Use official package names** from trusted registries
4. **Contact maintainers** if you believe this is a false positive

## Platform-Specific Issues

### Windows-Specific

#### PowerShell Execution Policy
**Symptoms:**
```
Execution of scripts is disabled on this system
```

**Solution:**
```powershell
# Run as Administrator
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### Windows Defender / Antivirus
**Symptoms:**
- MCP Helper binary gets quarantined
- Installation processes are blocked

**Solutions:**
1. **Add exclusions** for MCP Helper directory
2. **Temporarily disable** real-time protection during installation
3. **Download from trusted sources** only

#### Long Path Support
**Symptoms:**
```
Path too long error during installation
```

**Solution:**
```powershell
# Enable long path support (Windows 10 version 1607+)
# Run as Administrator
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### macOS-Specific

#### Gatekeeper Issues
**Symptoms:**
```
"mcp" cannot be opened because it is from an unidentified developer
```

**Solutions:**
1. **Right-click → Open** (one-time bypass)
2. **Remove quarantine attribute**:
   ```bash
   xattr -d com.apple.quarantine mcp
   ```
3. **Allow in System Preferences** → Security & Privacy

#### Homebrew Conflicts
**Symptoms:**
- Multiple Node.js versions causing conflicts
- Permission issues with global npm packages

**Solutions:**
```bash
# Clean up Homebrew
brew doctor
brew cleanup

# Fix permissions
sudo chown -R $(whoami) $(brew --prefix)/*

# Use Homebrew's Node.js consistently
brew unlink node && brew link node
```

### Linux-Specific

#### Package Manager Conflicts
**Symptoms:**
- Multiple Node.js installations
- Permission issues with global packages

**Solutions:**
```bash
# Use system package manager consistently
sudo apt purge nodejs npm  # Remove all versions
sudo apt install nodejs npm  # Reinstall clean

# Or use nvm for version management
```

#### SELinux Issues
**Symptoms:**
```
Permission denied errors despite correct file permissions
```

**Solutions:**
```bash
# Check SELinux status
getenforce

# Temporarily disable (not recommended for production)
sudo setenforce 0

# Or configure SELinux policies appropriately
```

## Advanced Troubleshooting

### Log Files and Debug Information

#### MCP Helper Logs
```bash
# Run with maximum verbosity
RUST_LOG=debug mcp install server-name --verbose

# Environment variables for debugging
RUST_BACKTRACE=1 mcp install server-name  # Show stack traces
```

#### Client Logs
- **Claude Desktop**: Check application logs in system log viewers
- **VS Code**: Check Output panel → "MCP" or "GitHub Copilot"
- **Cursor**: Check developer console

#### System Logs
```bash
# macOS
Console.app or: tail -f /var/log/system.log

# Linux
journalctl -f
tail -f /var/log/syslog

# Windows
Event Viewer → Windows Logs → Application
```

### Configuration File Debugging

#### Validate JSON Configuration
```bash
# Check JSON syntax
python -m json.tool config.json

# Or use jq
jq '.' config.json

# Pretty print and validate
cat config.json | jq .
```

#### Manual Configuration
If automatic configuration fails, you can manually edit client configs:

**Claude Desktop config example**:
```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "/path/to/allowed/dir"],
      "env": {
        "NODE_ENV": "production"
      }
    }
  }
}
```

### Performance Issues

#### Slow Installation
**Causes & Solutions:**
1. **Slow network**: Use `--verbose` to identify bottlenecks
2. **Large packages**: This is normal for Docker images
3. **Dependency resolution**: NPM can be slow; consider `npm config set registry https://registry.npmjs.org/`

#### High Memory Usage
**Solutions:**
1. **Close other applications** during installation
2. **Use `--dry-run`** first to estimate resource needs
3. **Install servers one at a time** instead of batch installation

## Getting Help

### Before Asking for Help
1. **Check this troubleshooting guide**
2. **Run with `--verbose` flag** and include output
3. **Test with `--dry-run`** to isolate issues  
4. **Check system requirements** and dependencies

### Where to Get Help
1. **GitHub Issues**: https://github.com/sapientpants/mcp-helper/issues
2. **Include in your report**:
   - Operating system and version
   - MCP Helper version (`mcp --version`)
   - Full command used
   - Complete error output (with `--verbose`)
   - System information (Node.js version, Docker version, etc.)

### Useful Commands for Bug Reports
```bash
# System information
mcp --version
node --version 2>/dev/null || echo "Node.js not found"
docker --version 2>/dev/null || echo "Docker not found"
python3 --version 2>/dev/null || echo "Python not found"

# Platform detection
uname -a  # Linux/macOS
systeminfo | findstr /B /C:"OS Name" /C:"OS Version"  # Windows

# Detailed error output
mcp install problematic-server --verbose --dry-run
```

### Creating Minimal Reproduction Cases
When reporting bugs:
1. **Use the simplest possible example** that shows the problem
2. **Test with official MCP servers** first (like `@modelcontextprotocol/server-filesystem`)
3. **Include exact steps** to reproduce the issue
4. **Test on a clean system** if possible

This troubleshooting guide should help resolve most common issues with MCP Helper!