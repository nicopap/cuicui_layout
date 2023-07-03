check:
	cargo clippy --workspace
run:
	cargo test
	# RUST_BACKTRACE=1 cargo run --bin ui_debug
	# cd examples/ui_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin ui_debug -- -Z macro-backtrace 
	# cd examples/ui_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin ui_debug -- -Zunpretty=expanded
