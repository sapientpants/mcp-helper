# MCP Server Registry Design

## Overview

The MCP server registry is a simple JSON file hosted on GitHub that contains metadata about known MCP servers. The CLI will download this file on first use and cache it locally, with options to update on demand.

## Registry Structure

### Registry Location
- **GitHub Repository**: `mcp-helper/mcp-server-registry` (or similar)
- **File**: `registry.json`
- **Raw URL**: `https://raw.githubusercontent.com/mcp-helper/mcp-server-registry/main/registry.json`

### JSON Structure

```json
{
  "version": "1.0.0",
  "updated": "2024-01-15T00:00:00Z",
  "servers": {
    "@modelcontextprotocol/server-filesystem": {
      "type": "npm",
      "runtime": "node",
      "minVersion": "18.0.0",
      "description": "File system access for MCP",
      "homepage": "https://github.com/modelcontextprotocol/servers",
      "config": {
        "required": ["allowedDirectories"],
        "optional": ["readOnly"],
        "defaults": {
          "allowedDirectories": ["~/Documents", "~/Downloads"]
        }
      }
    },
    "@modelcontextprotocol/server-github": {
      "type": "npm",
      "runtime": "node", 
      "minVersion": "18.0.0",
      "description": "GitHub API access for MCP",
      "config": {
        "required": ["token"],
        "optional": ["org", "repo"],
        "prompts": {
          "token": {
            "message": "GitHub Personal Access Token",
            "type": "password"
          },
          "org": {
            "message": "Default organization (optional)",
            "type": "text"
          }
        }
      }
    },
    "mcp-server-sqlite": {
      "type": "npm",
      "runtime": "node",
      "minVersion": "18.0.0",
      "description": "SQLite database access for MCP",
      "config": {
        "required": ["databasePath"],
        "optional": ["readOnly"]
      }
    },
    "mcp-server-fetch": {
      "type": "python",
      "runtime": "python",
      "minVersion": "3.10",
      "package": "mcp-server-fetch",
      "description": "Web fetching capabilities for MCP",
      "config": {
        "optional": ["userAgent", "timeout"]
      }
    },
    "rust-docs-mcp-server": {
      "type": "binary",
      "runtime": "native",
      "description": "Rust documentation server for MCP",
      "platforms": {
        "darwin-arm64": {
          "url": "https://github.com/Govcraft/rust-docs-mcp-server/releases/download/v1.0.0/server-aarch64-apple-darwin",
          "sha256": "abc123..."
        },
        "darwin-x64": {
          "url": "https://github.com/Govcraft/rust-docs-mcp-server/releases/download/v1.0.0/server-x86_64-apple-darwin",
          "sha256": "def456..."
        },
        "linux-x64": {
          "url": "https://github.com/Govcraft/rust-docs-mcp-server/releases/download/v1.0.0/server-x86_64-unknown-linux-gnu",
          "sha256": "ghi789..."
        },
        "win32-x64": {
          "url": "https://github.com/Govcraft/rust-docs-mcp-server/releases/download/v1.0.0/server-x86_64-pc-windows-msvc.exe",
          "sha256": "jkl012..."
        }
      }
    },
    "postgres-mcp-server": {
      "type": "docker",
      "runtime": "docker",
      "image": "postgres/mcp-server:latest",
      "description": "PostgreSQL database access for MCP",
      "config": {
        "required": ["DATABASE_URL"],
        "optional": ["SCHEMA", "SSL_MODE"]
      }
    }
  }
}
```

## CLI Implementation

### Registry Management

```rust
// Location: src/registry/mod.rs

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/mcp-helper/mcp-server-registry/main/registry.json";
const CACHE_FILE: &str = "server-registry.json";
const CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours

pub struct ServerRegistry {
    cache_dir: PathBuf,
    registry: Option<RegistryData>,
}

impl ServerRegistry {
    pub fn new() -> Result<Self> {
        let cache_dir = directories::ProjectDirs::from("dev", "mcp-helper", "mcp")
            .context("Failed to determine cache directory")?
            .cache_dir()
            .to_path_buf();
        
        Ok(Self {
            cache_dir,
            registry: None,
        })
    }
    
    pub fn load(&mut self) -> Result<()> {
        let cache_file = self.cache_dir.join(CACHE_FILE);
        
        // Try to load from cache first
        if cache_file.exists() {
            let metadata = fs::metadata(&cache_file)?;
            let age = metadata.modified()?.elapsed().unwrap_or(CACHE_TTL);
            
            if age < CACHE_TTL {
                let data = fs::read_to_string(&cache_file)?;
                self.registry = Some(serde_json::from_str(&data)?);
                return Ok(());
            }
        }
        
        // Cache is missing or stale, download fresh copy
        self.update()?;
        Ok(())
    }
    
    pub fn update(&mut self) -> Result<()> {
        println!("📥 Downloading server registry...");
        
        let response = reqwest::blocking::get(REGISTRY_URL)
            .context("Failed to download registry")?;
        
        if !response.status().is_success() {
            bail!("Failed to download registry: {}", response.status());
        }
        
        let data = response.text()?;
        let registry: RegistryData = serde_json::from_str(&data)
            .context("Failed to parse registry")?;
        
        // Save to cache
        fs::create_dir_all(&self.cache_dir)?;
        let cache_file = self.cache_dir.join(CACHE_FILE);
        fs::write(&cache_file, &data)?;
        
        self.registry = Some(registry);
        println!("✓ Registry updated successfully");
        Ok(())
    }
    
    pub fn get_server(&self, name: &str) -> Option<&ServerMetadata> {
        self.registry.as_ref()?.servers.get(name)
    }
}
```

### CLI Commands

```bash
# Update registry manually
mcp registry update

# Show registry info
mcp registry info

# Search registry
mcp registry search <query>

# Install uses registry automatically
mcp install @modelcontextprotocol/server-filesystem
```

### Usage in Install Command

```rust
// In src/main.rs or install handler

fn install_server(name: &str, options: InstallOptions) -> Result<()> {
    // Load registry (downloads if needed)
    let mut registry = ServerRegistry::new()?;
    registry.load()?;
    
    // Check if server is in registry
    if let Some(metadata) = registry.get_server(name) {
        // Use registry metadata
        match metadata.server_type {
            ServerType::Npm => install_npm_server(name, metadata)?,
            ServerType::Binary => install_binary_server(name, metadata)?,
            ServerType::Python => install_python_server(name, metadata)?,
            ServerType::Docker => install_docker_server(name, metadata)?,
        }
    } else {
        // Fall back to detection heuristics
        println!("⚠️  Server '{}' not found in registry", name);
        println!("Attempting automatic detection...");
        detect_and_install(name)?;
    }
    
    Ok(())
}
```

## Registry Update Flow

```
$ mcp install @modelcontextprotocol/server-filesystem

🔍 Checking server registry...
📥 Registry is 3 days old, downloading update...
✓ Registry updated (247 servers available)

📦 Installing @modelcontextprotocol/server-filesystem
✓ Type: NPM package
✓ Requires: Node.js >=18.0.0
✓ Configuration: allowedDirectories (required)
```

## Contributing to Registry

Users can contribute by submitting PRs to the registry repository:

```markdown
# Adding a New Server

1. Fork the mcp-server-registry repository
2. Edit registry.json to add your server
3. Ensure all required fields are present
4. Submit a pull request

## Required Fields
- type: npm|binary|python|docker
- runtime: node|python|docker|native
- description: Brief description of the server

## Optional Fields
- minVersion: Minimum runtime version
- config: Configuration schema
- platforms: Platform-specific binaries
```

## Cache Management

### Cache Location
- **Windows**: `%LOCALAPPDATA%\mcp-helper\cache\server-registry.json`
- **macOS**: `~/Library/Caches/mcp-helper/server-registry.json`
- **Linux**: `~/.cache/mcp-helper/server-registry.json`

### Cache Strategy
- Download on first use
- Cache for 24 hours
- Update on `mcp registry update`
- Update if cache is corrupted
- Work offline with cached data

## Benefits

1. **Simple** - Just a JSON file, easy to understand and edit
2. **Version Controlled** - Full history of changes on GitHub
3. **Community Driven** - Anyone can submit PRs to add servers
4. **Offline Support** - Works with cached data
5. **Fast** - No API rate limits or authentication needed
6. **Transparent** - Users can see exactly what data is stored

## Future Enhancements

1. **Multiple Registries** - Support custom/private registries
2. **Registry Validation** - GitHub Actions to validate PRs
3. **Auto-generation** - Script to scan known servers and update
4. **Versioning** - Support for multiple server versions
5. **Categories/Tags** - Better organization of servers
6. **Stats Tracking** - Anonymous usage statistics (opt-in)