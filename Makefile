.PHONY: all amd64-bin amd64-deb arm-bin arm-deb copy-builds

VERSION = 2.3.0
BUILD_DIR = build

all: amd64-bin amd64-deb arm-bin arm-deb copy-builds

amd64-bin:
	nix build -o amd64-bin

amd64-deb:
	nix build .#deb -o amd64-deb

arm-bin:
	nix build --system aarch64-linux -o arm-bin

arm-deb:
	nix build .#deb --system aarch64-linux -o arm-deb

copy-builds:
	mkdir -p $(BUILD_DIR)
	cp amd64-bin/bin/h8r $(BUILD_DIR)/h8r_$(VERSION)_amd64
	cp amd64-deb/h8r.deb $(BUILD_DIR)/h8r_$(VERSION)_amd64.deb
	cp arm-bin/bin/h8r $(BUILD_DIR)/h8r_$(VERSION)_arm64
	cp arm-deb/h8r.deb $(BUILD_DIR)/h8r_$(VERSION)_arm64.deb
