## rEFInd-GUI

rEFInd-GUI is a Rust workspace for managing rEFInd boot entries, system configuration, and boot environment tooling. The project is currently in a CLI-first stage, with a core library and GUI shell scaffolding ready for further implementation.

### Features
- CLI for system inspection and boot management
- Core data model for distro detection, kernel scanning, ESP management, and secure-boot checks
- Workspace structure prepared for GUI expansion
- Test coverage for the core detection logic

### Current status
- Buildable Rust workspace
- CLI smoke-tested with help output
- Linux OS detection and system metadata collection implemented at the library layer

### Usage

```bash
source "$HOME/.cargo/env"
cargo build --workspace
cargo run -p refind-cli -- --help
```

### Project layout
- `refind-cli/` — user-facing command-line tool
- `refind-core/` — shared logic and models
- `refind-gui/` — GUI layer scaffold

### Roadmap
- Real ESP discovery and kernel detection
- Actual rEFInd config parsing and editing
- GUI implementation for Linux and Windows support
