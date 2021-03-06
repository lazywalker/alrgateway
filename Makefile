VER=$(shell cargo pkgid | cut -d\# -f2)
GIT_VER=$(shell git rev-parse --short HEAD)
APP_VERSION=${VER}.${GIT_VER}
TARBALL="target/tarball"
APP_NAME="alrgateway"

all: debug

ver:
	@echo Version: ${APP_NAME} v${APP_VERSION}

debug:
	cargo build

macos:
	cargo build --release --target x86_64-apple-darwin

armv7:
	CC_armv7_unknown_linux_musleabihf=armv7-unknown-linux-musleabihf-gcc \
	CXX_armv7_unknown_linux_musleabihf=armv7-unknown-linux-musleabihf-g++ \
	AR_armv7_unknown_linux_musleabihf=armv7-unknown-linux-musleabihf-ar \
	CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=armv7-unknown-linux-musleabihf-gcc \
	cargo build --release --target armv7-unknown-linux-musleabihf

linux:
	cargo build --release --target x86_64-unknown-linux-musl

clean:
	cargo clean

tarball: macos linux armv7
	@echo Creating tarball...
	@mkdir -p ${TARBALL}
	
	@echo Creating x86_64-apple-darwin
	@tar cvfz "${TARBALL}/${APP_NAME}-${APP_VERSION}-x86_64-apple-darwin.tar.gz" -C target/x86_64-apple-darwin/release/ ${APP_NAME} 

	@echo Creating x86_64-unknown-linux-musl
	@tar cvfz "${TARBALL}/${APP_NAME}-${APP_VERSION}-x86_64-unknown-linux-musl.tar.gz" -C target/x86_64-unknown-linux-musl/release/ ${APP_NAME}

	@echo Creating armv7-unknown-linux-musleabihf
	@tar cvfz "${TARBALL}/${APP_NAME}-${APP_VERSION}-armv7-unknown-linux-musleabihf.tar.gz" -C target/armv7-unknown-linux-musleabihf/release/ ${APP_NAME}