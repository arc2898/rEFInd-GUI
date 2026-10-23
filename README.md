## rEFInd-GUI

rEFInd-GUI is a Rust workspace for managing rEFInd boot entries, system configuration, and boot environment tooling. The project is currently in a CLI-first stage, with a core library and GUI shell scaffolding ready for further implementation.

### Features
- Rust CLI foundations for system inspection and boot configuration diagnostics
- Read-only Linux ESP discovery using the EFI System Partition type, with a fallback for FAT ESPs mounted at common paths
- Read-only refind.conf inspection that preserves comments and directives the tool does not interpret
- Core models for Linux distribution, kernel, and Secure Boot information
- GTK GUI shell scaffold for future expansion

### Current status
- Linux ESP discovery uses lsblk and reports command errors instead of treating them as an empty scan
- config show displays the selected file as written; --config <path> selects a file explicitly
- config validate parses timeout, default selection, and showtools settings, and reports malformed or unsafe timeout values
- Validation is partial: it does not interpret every rEFInd directive or prove that a boot entry will work
- Configuration commands do not modify the file

### Usage

~~~bash
source "$HOME/.cargo/env"
cargo build --workspace
cargo run -p refind-cli -- --help
cargo run -p refind-cli -- config show
cargo run -p refind-cli -- --config /path/to/refind.conf config validate
~~~

Without --config, config commands look in common Linux locations. If none is found, they return an error instead of silently validating an empty default config.

### Project layout
- `refind-cli/` — user-facing command-line tool
- `refind-core/` — shared logic and models
- `refind-gui/` — GUI layer scaffold

### Roadmap
- Broader kernel and initramfs discovery across Linux distributions
- Semantic validation for more rEFInd directives
- Safe config preview, backup, and edit workflow
- GUI implementation for Linux and Windows support

