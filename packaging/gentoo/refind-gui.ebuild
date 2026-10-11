EAPI=8

inherit cargo

DESCRIPTION="rEFInd management utility and boot environment tooling"
HOMEPAGE="https://github.com/rEFInd-GUI/rEFInd-GUI"
SRC_URI="https://github.com/rEFInd-GUI/rEFInd-GUI/archive/refs/tags/v${PV}.tar.gz"

LICENSE="MIT Apache-2.0"
SLOT="0"
KEYWORDS="~amd64"
IUSE=""

DEPEND=">=virtual/rust-1.78.0"
RDEPEND=""
BDEPEND="${DEPEND}"

src_compile() {
  cargo build --release --locked --bin refind-cli || die "cargo build failed"
}

src_install() {
  install -Dm755 target/release/refind-cli "${D}/usr/bin/refind-cli" || die "install failed"
  dodoc README.md
}
