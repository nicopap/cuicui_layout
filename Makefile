CLIPPY_ARGS=-- --deny clippy::all --deny clippy::pedantic --deny clippy::nursery \
	--warn clippy::needless-pass-by-value \
	--allow clippy::use-self
.PHONY: check run pre-hook

check:
	cargo clippy  $(CLIPPY_ARGS)

run:
	# cargo test -p parse_dsl_macro # --features cuicui_chirp/trace_parser
	RUST_BACKTRACE=1 cargo run -p sprite_debug --features cuicui_layout/debug
	# cd examples/sprite_debug && RUSTC_BOOTSTRAP=1 cargo rustc -p sprite_debug -- -Zunpretty=expanded
	# cd examples/bevypunk && RUSTC_BOOTSTRAP=1 cargo rustc -p bevypunk -- -Z macro-backtrace

pre-hook:
	cargo test --package cuicui_dsl --features test_and_doc
	cargo test --package cuicui_chirp --features test_and_doc
	cargo test --workspace --exclude cuicui_dsl
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
	cargo clippy --workspace $(CLIPPY_ARGS)
	cargo fmt --all -- --check
	cargo clippy --no-default-features --package cuicui_layout_bevy_ui $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout_bevy_sprite $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout --features debug $(CLIPPY_ARGS)
