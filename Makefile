check:
	cargo clippy --workspace
run:
	# cargo test
	RUST_BACKTRACE=1 cargo run --bin ui_debug
