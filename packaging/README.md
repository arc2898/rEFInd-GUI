# Packaging matrix for rEFInd-GUI

This project is intentionally designed for distro-native packaging rather than sandboxed app formats. A boot manager is OS-level infrastructure, so Arch, SUSE/openSUSE, Gentoo, and Void are the most appropriate targets.

The Flatpak and Snap formats were intentionally left out here because they are not a good fit for EFI-boot management, firmware access, and system-level installs.

## Included package families

- Arch Linux: `arch/PKGBUILD`
- SUSE / openSUSE: `suse/refind-gui.spec`
- Gentoo: `gentoo/refind-gui.ebuild`
- Void Linux: `void/template`

## Build assumptions

- The shipped binary is `refind-cli`.
- The package should compile from the workspace root with `cargo build --release --locked --bin refind-cli`.
- Version alignment should follow the Cargo workspace version in the root `Cargo.toml` file.

## Status

- Linux build verified with `cargo test --workspace --all-targets`
- Windows cross-build verified with `cargo check --target x86_64-pc-windows-gnu --workspace`
- Packaging templates prepared for distro-native distribution workflows
