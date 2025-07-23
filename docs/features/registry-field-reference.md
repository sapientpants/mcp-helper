# MCP Server Registry Field Reference

## Top-Level Fields

### `version`
- **Type**: String (semver)
- **Required**: Yes
- **Purpose**: Version of the registry schema itself (not server versions)
- **Example**: `"1.0.0"`
- **Usage**: Allows CLI to handle schema changes gracefully

### `updated`
- **Type**: ISO 8601 timestamp
- **Required**: Yes
- **Purpose**: When the registry was last modified
- **Example**: `"2024-01-15T00:00:00Z"`
- **Usage**: Helps users know how current the data is

### `servers`
- **Type**: Object (map of server names to metadata)
- **Required**: Yes
- **Purpose**: Contains all server definitions
- **Example**: `{ "@modelcontextprotocol/server-filesystem": {...} }`

## Server Metadata Fields

Each server entry contains:

### `type`
- **Type**: Enum string
- **Required**: Yes
- **Values**: `"npm"`, `"binary"`, `"python"`, `"docker"`
- **Purpose**: Tells the CLI how to install/run the server
- **Example**: `"npm"`
- **Usage**:
  - `npm`: Use `npx` to run
  - `binary`: Download platform-specific executable
  - `python`: Use Python interpreter
  - `docker`: Use Docker to run container

### `runtime`
- **Type**: Enum string
- **Required**: Yes
- **Values**: `"node"`, `"python"`, `"docker"`, `"native"`
- **Purpose**: What runtime environment is needed
- **Example**: `"node"`
- **Usage**: Determines which dependency to check (Node.js, Python, Docker, or none)

### `minVersion`
- **Type**: String (semver)
- **Required**: No (only for node/python)
- **Purpose**: Minimum required version of the runtime
- **Example**: `"18.0.0"` for Node.js, `"3.10"` for Python
- **Usage**: CLI checks if installed version meets this requirement

### `description`
- **Type**: String
- **Required**: Yes
- **Purpose**: Brief description of what the server does
- **Example**: `"File system access for MCP"`
- **Usage**: Shown in search results and install confirmation

### `homepage`
- **Type**: URL string
- **Required**: No
- **Purpose**: Link to server's documentation or repository
- **Example**: `"https://github.com/modelcontextprotocol/servers"`
- **Usage**: Helps users find more information

### `package` (Python only)
- **Type**: String
- **Required**: Yes for Python servers
- **Purpose**: PyPI package name (may differ from server name)
- **Example**: `"mcp-server-fetch"`
- **Usage**: Used with `pip install` or `uvx`

### `image` (Docker only)
- **Type**: String
- **Required**: Yes for Docker servers
- **Purpose**: Docker image name and tag
- **Example**: `"postgres/mcp-server:latest"`
- **Usage**: Used with `docker run`

## Configuration Schema Fields

### `config`
- **Type**: Object
- **Required**: No
- **Purpose**: Describes server configuration requirements
- **Contains**: `required`, `optional`, `defaults`, `prompts`

### `config.required`
- **Type**: Array of strings
- **Required**: No
- **Purpose**: Configuration fields that must be provided
- **Example**: `["token", "apiKey"]`
- **Usage**: CLI will prompt for these during installation

### `config.optional`
- **Type**: Array of strings
- **Required**: No
- **Purpose**: Configuration fields that may be provided
- **Example**: `["timeout", "retries"]`
- **Usage**: CLI may prompt with option to skip

### `config.defaults`
- **Type**: Object (field name to default value)
- **Required**: No
- **Purpose**: Default values for configuration fields
- **Example**: `{ "allowedDirectories": ["~/Documents", "~/Downloads"] }`
- **Usage**: Pre-fills prompts, user can accept or modify

### `config.prompts`
- **Type**: Object (field name to prompt definition)
- **Required**: No
- **Purpose**: Customize how CLI prompts for each field
- **Example**:
  ```json
  {
    "token": {
      "message": "GitHub Personal Access Token",
      "type": "password"
    }
  }
  ```
- **Usage**: Controls prompt text and input type (text/password)

## Platform-Specific Fields (Binary servers)

### `platforms`
- **Type**: Object (platform key to download info)
- **Required**: Yes for binary servers
- **Purpose**: Platform-specific download information
- **Platform Keys**: 
  - `"darwin-arm64"` (Apple Silicon Mac)
  - `"darwin-x64"` (Intel Mac)
  - `"linux-x64"` (64-bit Linux)
  - `"linux-arm64"` (ARM64 Linux)
  - `"win32-x64"` (64-bit Windows)

### `platforms.<platform>.url`
- **Type**: URL string
- **Required**: Yes
- **Purpose**: Direct download URL for the binary
- **Example**: `"https://github.com/owner/repo/releases/download/v1.0.0/server-darwin-arm64"`
- **Usage**: CLI downloads from this URL

### `platforms.<platform>.sha256`
- **Type**: String (hex)
- **Required**: No (but recommended)
- **Purpose**: SHA256 checksum for integrity verification
- **Example**: `"abc123def456..."`
- **Usage**: CLI verifies download hasn't been tampered with

## Example Breakdown

Here's a complete example with all fields explained:

```json
{
  "@modelcontextprotocol/server-github": {
    // How to install/run this server
    "type": "npm",
    
    // What runtime is needed
    "runtime": "node",
    
    // Minimum Node.js version
    "minVersion": "18.0.0",
    
    // What this server does
    "description": "GitHub API access for MCP",
    
    // Where to find more info
    "homepage": "https://github.com/modelcontextprotocol/servers",
    
    // Configuration schema
    "config": {
      // Must have these fields
      "required": ["token"],
      
      // Can have these fields
      "optional": ["org", "repo"],
      
      // Default values
      "defaults": {
        "org": "modelcontextprotocol"
      },
      
      // How to prompt for each field
      "prompts": {
        "token": {
          "message": "Enter your GitHub Personal Access Token",
          "type": "password"  // Hides input
        },
        "org": {
          "message": "Default organization (press Enter to skip)",
          "type": "text"
        }
      }
    }
  }
}
```

## Why These Fields?

1. **type/runtime**: Essential for knowing how to install and run
2. **minVersion**: Prevents cryptic errors from incompatible versions
3. **description**: Helps users understand what they're installing
4. **config schema**: Enables interactive, guided configuration
5. **platforms**: Supports truly cross-platform binary distribution
6. **defaults**: Improves UX by suggesting sensible values
7. **prompts**: Allows security-conscious input (passwords)
8. **sha256**: Ensures binary integrity and security

This structure balances simplicity with completeness, providing all necessary information while remaining human-readable and easy to maintain.