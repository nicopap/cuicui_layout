.PHONY: check run pre-hook

check:
	cargo clippy -- --deny clippy::all

run:
	# cargo test
	RUST_BACKTRACE=1 cargo run --bin sprite_mesh_debug
	# cd examples/sprite_mesh_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin sprite_mesh_debug -- -Zunpretty=expanded
	# cd examples/ui_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin ui_debug -- -Z macro-backtrace

pre-hook:
	cargo test
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
	cargo clippy --workspace -- --deny clippy::all
	cargo fmt --all -- --check
	cargo clippy --no-default-features --package cuicui_layout_bevy_ui -- --deny clippy::all
	cargo clippy --no-default-features --package cuicui_layout_bevy_sprite -- --deny clippy::all
	cargo clippy --no-default-features --package cuicui_layout -- --deny clippy::all
