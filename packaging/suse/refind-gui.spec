Name: refind-gui
Version: 0.1.0
Release: 1%{?dist}
Summary: rEFInd management utility and boot environment tooling
License: MIT OR Apache-2.0
URL: https://github.com/rEFInd-GUI/rEFInd-GUI
Source0: https://github.com/rEFInd-GUI/rEFInd-GUI/archive/refs/tags/v%{version}.tar.gz
BuildRequires: cargo, gcc, make, rust
Requires: glibc

%description
A command-line and library-based toolkit for inspecting and managing rEFInd boot configurations, kernel state, and ESP mounts.

%prep
%autosetup -n rEFInd-GUI-%{version}

%build
cargo build --release --locked --bin refind-cli

%install
install -Dm755 target/release/refind-cli %{buildroot}%{_bindir}/refind-cli
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md

%files
%{_bindir}/refind-cli
%{_docdir}/%{name}/README.md

%changelog
* Sat Oct 11 2026 - package template generated for rEFInd-GUI
- Initial package template for SUSE/openSUSE builds
