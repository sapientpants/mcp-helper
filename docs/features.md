# MCP Helper - Feature Planning Document

> **Note**: This is an internal planning document that outlines the core features and design philosophy of MCP Helper. 
> For user-facing documentation, see [README.md](../README.md).

## Overview

MCP Helper is a cross-platform tool designed to eliminate compatibility issues when working with Model Context Protocol (MCP) servers. This document outlines the core features and scope boundaries.

## Core Features - Pure Cross-Platform Compatibility

### 1. Universal MCP Launcher

```bash
mcp run <server-name>     # Handles npx/path issues transparently
mcp install <server-name> # Works identically on Windows/Mac/Linux
```

**Key capabilities:**
- Windows npx wrapper built-in - no manual CMD fixes needed
- Path translation - automatically converts `/` to `\` and handles escaping
- Environment inheritance - fixes GUI application env variable issues

### 2. Configuration File Management

**Auto-detects where each MCP client stores configs:**
- Windows: `%APPDATA%\Claude\`
- macOS: `~/Library/Application Support/Claude/`
- Linux: `~/.config/Claude/`

**Unified commands that work everywhere:**
```bash
mcp config add my-server      # Adds to correct location automatically
mcp config list               # Shows all configured servers
mcp config show my-server     # Display server configuration details
mcp config update my-server   # Update existing server configuration
mcp config remove my-server   # Remove server configuration
```

### 3. Development Environment Normalization

- Node.js version compatibility layer
- Package manager abstraction (npm/yarn/pnpm differences)
- IDE configuration generators (VS Code, IntelliJ, etc.)

```bash
mcp init                    # Sets up project with right configs for your OS
mcp generate-ide-config     # Creates .vscode/settings.json that works
```

### 4. Path and Environment Utilities

- Path converters for config files
- Environment variable helpers

```bash
# Developers never need to think about platform differences
mcp env set API_KEY=value   # Handles platform-specific storage
mcp path convert ./my/path  # Returns platform-appropriate path
```

### 5. Installation Verification

- Compatibility checker before installation
- Dependency validator for platform-specific requirements

```bash
mcp doctor                  # "npx is not in PATH on Windows"
mcp verify my-server        # "This server requires Unix paths"
```

## What This Does NOT Include

- ❌ Security features (authentication, authorization)
- ❌ Runtime monitoring or resource limits
- ❌ Policy enforcement or audit logging
- ❌ Network traffic inspection
- ❌ Marketplace or registry features

## The Focused Value Proposition

This is the "Make MCP Just Work™" tool:

- Install it once, and MCP servers work the same on any OS
- No more debugging Windows-specific issues
- No more "works on my machine" problems
- No overlap with security/runtime concerns

Think of it like `nvm` (Node Version Manager) - it doesn't do security or monitoring, it just makes Node.js work consistently everywhere. This does the same for MCP.

## MVP: The Minimum Lovable Product

Just three commands that solve 90% of the pain:

```bash
mcp setup                   # One-time fix for your OS
mcp install <server>        # Actually works on Windows
mcp run <server>            # No path escaping needed
```

That's it. Pure developer experience improvement, no feature creep into security or operations territory.