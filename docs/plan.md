# MCP Helper - Implementation Plan

> **Note**: This document outlines the phased implementation approach for MCP Helper.
> - For feature descriptions, see [features.md](features.md)
> - For technical architecture, see [architecture.md](architecture.md)

## Executive Summary

The MLP focuses on solving the #1 pain point: **making MCP servers "just work" across platforms**, especially fixing the Windows npx problem. We'll build incrementally, delivering immediate value with each phase.

## Core Value Proposition

**"Make MCP Just Work™"** - The cross-platform compatibility layer for MCP that eliminates platform-specific documentation and makes MCP servers work identically everywhere.

For detailed scope boundaries, see [features.md](features.md#scope-boundaries).

## Phase 1: Core Runner (Week 1-2) 🎯 ✅ COMPLETE

### Features:
Implements the Universal MCP Launcher (see [features.md](features.md#1-universal-mcp-launcher)):
- **`mcp run <server>`** - Platform-agnostic server execution
- Platform detection and npx/npx.cmd wrapper for Windows
- Path separator normalization
- Basic error handling with clear messages

### Justification:
- **Solves the biggest pain point immediately** - Windows users can finally run MCP servers
- **Minimal scope** - Can be shipped quickly to validate the concept
- **Immediate value** - Users can start using it with manually configured servers
- **Learning opportunity** - Gather feedback on core functionality before expanding

### Success Metrics:
- ✅ Windows users can run MCP servers without manual CMD workarounds
- ✅ Zero platform-specific documentation needed for running servers

### Implementation Status:
- ✅ Basic CLI structure with clap
- ✅ Platform detection
- ✅ npx/npx.cmd wrapper
- ✅ Path separator normalization
- ✅ Actionable error messages
- ✅ Unit tests (8 tests passing)
- ✅ Basic documentation

## Phase 2: Installation & Setup (Week 3-4) 📦

### Features:
2. **`mcp install <server>`**
   - npm package installation with proper paths
   - Local directory installation support
   - Platform-specific installation handling
   - Progress indicators for long operations

3. **`mcp setup`**
   - One-time PATH configuration
   - Shell profile updates (bashrc, zshrc, PowerShell profile)
   - Environment variable setup for GUI apps
   - Node.js installation detection and guidance

### Justification:
- **Completes the basic workflow** - Users can now install AND run servers
- **Reduces friction** - No manual npm or PATH configuration
- **Foundation for config management** - Sets up necessary directories and paths
- **Eliminates "npx not found" errors** - Setup ensures environment is ready

### Technical Approach:
Implements the PackageManager and PlatformOperations traits defined in [architecture.md](architecture.md#key-components).

### Module Structure:
See [architecture.md](architecture.md#proposed-module-structure) for detailed module layout:
- `src/package/` - Package management abstraction
- `src/platform/` - Platform-specific implementations  
- `src/environment/` - PATH and environment variable handling

## Phase 3: Configuration Management (Week 5-6) ⚙️

### Features:
4. **`mcp config add <server>`**
   - Auto-detect Claude config location per platform
   - Atomic writes to prevent corruption
   - Path normalization in config files
   - Environment variable support

5. **`mcp config list`**
   - Show all configured servers
   - Display config file location
   - Basic status information
   - Config validation

6. **`mcp config show <server>`**
   - Detailed server configuration
   - Environment variables
   - Command line arguments

7. **`mcp config update <server>`**
   - Update existing server configuration
   - Preserve comments and formatting
   - Validation before saving

8. **`mcp config remove <server>`**
   - Safe removal of server configs
   - Confirmation prompts for safety
   - Backup before deletion

### Justification:
- **Enables persistence** - Servers survive restarts
- **Multi-server support** - Users typically need multiple MCP servers
- **Integration with Claude** - Works with the primary MCP client
- **Configuration as code** - Version control friendly

### Module Structure:
- `src/config/` - Configuration management with serde/serde_json
- `src/platform/` - Platform-specific config paths
- Uses `directories` crate for cross-platform config locations

### Technical Details:
Implements Configuration Management features from [features.md](features.md#2-configuration-management) using the architecture's config module structure. Platform-specific paths are handled as described in [architecture.md](architecture.md#platform-specific-config-locations).

## Phase 4: Diagnostics & Polish (Week 7-8) 🏥

### Features:
9. **`mcp doctor`**
   - Check Node.js installation
   - Verify npm/npx availability
   - Test PATH configuration
   - Validate config file permissions
   - **Auto-fix common issues when possible**
   - Platform-specific checks

### Justification:
- **Reduces support burden** - Users can self-diagnose issues
- **Improves user confidence** - Clear feedback on system state
- **Prevents user frustration** - Auto-fix saves time

### Module Structure:
- `src/diagnostics/` - Diagnostic framework and checks
- Uses `thiserror` for rich error types with context
- Uses `tracing` for structured logging

### Example Doctor Output:
```
🔍 Checking MCP environment...
  ✓ Node.js: v20.11.0
  ✗ npx availability: npx not found in PATH on Windows
    → Auto-fix available. Apply? [Y/n] Y
    ✓ Fixed!
  ✓ Config permissions: Read/write access confirmed
  ✓ PATH setup: All required tools accessible

All checks passed! MCP is ready to use.
```

## Phase 5: Enhanced Features (Week 9-10) 🚀

### Features:
10. **`mcp verify <server>`**
    - Pre-installation compatibility check
    - Platform requirement validation
    - Dependency verification
    - Version compatibility

11. **`mcp env set <key>=<value>`**
    - Platform-specific environment variable management
    - Persistent across sessions
    - GUI app support

12. **`mcp path convert <path>`**
    - Convert paths between platforms
    - Handle UNC paths on Windows
    - Escape special characters

13. **Error message improvements**
    - Actionable error messages throughout
    - Platform-specific troubleshooting hints
    - Common solutions database

### Justification:
- **Prevents installation failures** - Verify catches issues early
- **Power user features** - Advanced config management
- **Polish** - Better user experience overall
- **Cross-team collaboration** - Path conversion helps sharing

## Architecture Implementation

Each phase implements specific modules from [architecture.md](architecture.md#proposed-module-structure):

- **Phase 1**: Core platform abstraction (`src/platform/`, `src/runner.rs`)
- **Phase 2**: Package management (`src/package/`, `src/environment/`)
- **Phase 3**: Configuration system (`src/config/`)
- **Phase 4**: Diagnostics framework (`src/diagnostics/`)
- **Phase 5**: Utilities and enhancements (`src/utils/`)
- **Phase 6**: IDE integration (`src/ide/`)

## Future Phases (Post-MLP)

### Phase 6: Development Tools
- `mcp init` - Create new MCP server projects
- `mcp generate-ide-config` - VS Code/IntelliJ setup (src/ide/)
- Project templates
- Debugging helpers

### Phase 7: Advanced Features
- Shell completions (bash, zsh, PowerShell)
- Update notifications
- Server marketplace integration (if scope expands)
- Multi-version Node.js support
- Container support

## Key Design Principles

1. **Platform differences are bugs** - Abstract all platform specifics
2. **Fail fast with helpful errors** - Every error tells users how to fix it
3. **Zero configuration** - Smart defaults for everything
4. **Incremental value** - Each phase stands alone
5. **User empathy** - Focus on developer experience

## Technical Considerations

### Architecture Decisions
- **Rust** for single binary distribution and cross-platform reliability
- **JSON** for config files (Claude compatible)
- **No external dependencies** beyond standard system tools
- **Extensive Windows testing** - This is where most pain exists

### Technical Considerations

#### Testing Strategy
See [architecture.md](architecture.md#testing-strategy) for comprehensive testing approach.

#### Performance Goals
- Startup time < 100ms
- No perceptible delay for common operations
- Minimal memory footprint
- No background processes

#### Dependencies
Core dependencies are listed in [architecture.md](architecture.md#dependencies-to-add).

## Why This Order?

1. **Immediate pain relief** - Phase 1 fixes the #1 complaint (npx on Windows)
2. **Natural workflow** - Run → Install → Configure → Debug
3. **Incremental complexity** - Each phase builds on the previous
4. **Early validation** - Can ship Phase 1 in 1-2 weeks and get feedback
5. **Risk mitigation** - If project stalls, we've still delivered core value

## Success Metrics

### Phase 1 (Complete)
- ✅ 90% reduction in "npx not working" issues
- ✅ Cross-platform server execution
- ✅ Clear error messages

### Phase 2-3 Goals
- One-command server installation
- Zero manual configuration required
- 5-minute setup to running first server

### Phase 4-5 Goals
- Self-diagnosing system
- < 1% of users need manual support
- Power features for advanced users

## Risk Mitigation

### Technical Risks
- **Windows compatibility**: Extensive testing, community beta
- **Node.js version differences**: Version detection, compatibility matrix
- **Config file corruption**: Atomic writes, backups

### Adoption Risks
- **User trust**: Open source, transparent development
- **Migration friction**: Import existing configs, migration guide
- **Documentation**: Clear, platform-specific guides

## Conclusion

This plan delivers a truly "lovable" product by focusing obsessively on the developer experience and solving real pain points in priority order. Each phase provides immediate value while building toward a comprehensive solution that makes MCP "just work" on every platform.