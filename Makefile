RUST_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
CGIT_DIR := $(RUST_DIR)/../cgit

.PHONY: build test clean

build:
	cargo build --release --manifest-path $(RUST_DIR)/Cargo.toml
	ln -sf $(RUST_DIR)/target/release/cgit $(CGIT_DIR)/cgit

test: build
	$(MAKE) -C $(CGIT_DIR)/tests all

clean:
	cargo clean --manifest-path $(RUST_DIR)/Cargo.toml
	rm -f $(CGIT_DIR)/cgit
