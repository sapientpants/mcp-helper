# MCP Helper - Features

> **Note**: This document describes the user-facing features of MCP Helper.
> - For technical implementation details, see [architecture.md](architecture.md)
> - For the implementation timeline, see [plan.md](plan.md)
> - For user documentation, see [README.md](../README.md)

## Overview

MCP Helper is a cross-platform tool that makes Model Context Protocol (MCP) servers "just work" on any operating system. Think of it like `nvm` for MCP - it handles all platform-specific compatibility issues transparently.

## Core Features

### 1. Universal MCP Launcher

```bash
mcp run <server-name>     # Run any MCP server on any OS
mcp install <server-name> # Install servers with automatic compatibility
```

### 2. Configuration Management

```bash
mcp config add <server>     # Add server to MCP client config
mcp config list             # List all configured servers
mcp config show <server>    # Display server configuration
mcp config update <server>  # Update server configuration
mcp config remove <server>  # Remove server configuration
```

### 3. Development Tools

```bash
mcp init                    # Initialize new MCP server project
mcp generate-ide-config     # Generate IDE configuration files
```

### 4. Path and Environment Utilities

```bash
mcp env set KEY=value       # Set environment variables
mcp path convert <path>     # Convert paths between platforms
```

### 5. Diagnostics and Verification

```bash
mcp setup                   # One-time system setup
mcp doctor                  # Diagnose and fix common issues
mcp verify <server>         # Check server compatibility
```

## Scope Boundaries

MCP Helper focuses exclusively on cross-platform compatibility and does NOT include:

- ❌ Security features (authentication, authorization)
- ❌ Runtime monitoring or resource limits
- ❌ Policy enforcement or audit logging
- ❌ Network traffic inspection
- ❌ Marketplace or registry features

## Value Proposition

**"Make MCP Just Work™"** - Install it once, and MCP servers work identically on Windows, macOS, and Linux. No more platform-specific documentation, no more "works on my machine" problems.