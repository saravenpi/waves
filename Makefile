.PHONY: build release run clean help

help:
	@echo "Available targets:"
	@echo "  build   - Build the project in debug mode"
	@echo "  release - Build the project in release mode (optimized)"
	@echo "  run     - Build and run the project in debug mode"
	@echo "  clean   - Remove build artifacts"

build:
	cargo build
	cp target/debug/waves .

release:
	cargo build --release
	cp target/release/waves .

run:
	cargo run

clean:
	cargo clean
