.PHONY: check run pre-hook

check:
	cargo clippy --workspace

run: pre-hook
	# cargo test
	# RUST_BACKTRACE=1 cargo run --bin ui_debug
	# cd examples/sprite_mesh_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin sprite_mesh_debug -- -Zunpretty=expanded
	# cd examples/ui_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin ui_debug -- -Z macro-backtrace

pre-hook:
	cargo test
	cargo doc --workspace --no-deps
	cargo clippy --workspace -- --deny clippy::all
	cargo fmt --all -- --check
