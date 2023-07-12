CLIPPY_ARGS=-- --deny clippy::all --deny clippy::pedantic --deny clippy::nursery \
	--warn clippy::needless-pass-by-value \
	--allow clippy::use-self
.PHONY: check run pre-hook

check:
	cargo clippy $(CLIPPY_ARGS)
	cargo test --package cuicui_dsl --features test_and_doc

run:
	# cargo test
	RUST_BACKTRACE=1 cargo run --bin ui_debug
	# cd examples/sprite_mesh_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin sprite_mesh_debug -- -Zunpretty=expanded
	# cd examples/ui_debug && RUSTC_BOOTSTRAP=1 cargo rustc --bin ui_debug -- -Z macro-backtrace

pre-hook:
	cargo test --package cuicui_dsl --features test_and_doc
	cargo test --workspace --exclude cuicui_dsl
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
	cargo clippy --workspace $(CLIPPY_ARGS)
	cargo fmt --all -- --check
	cargo clippy --no-default-features --package cuicui_layout_bevy_ui $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout_bevy_sprite $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout $(CLIPPY_ARGS)
