CLIPPY_ARGS=-- -D clippy::all -D clippy::pedantic -D clippy::nursery \
	-D missing_docs \
	-D clippy::undocumented_unsafe_blocks \
	-W clippy::needless-pass-by-value \
	-A clippy::missing_const_for_fn \
	-A clippy::module_name_repetitions \
	-A clippy::redundant_pub_crate

.PHONY: check checkout-cyberpunk run pre-hook

examples/chirpunk/lunex-cyberpunk-assets:
	git clone --no-checkout --depth=1 --filter=tree:0 \
		https://github.com/IDEDARY/bevy-lunex-cyberpunk.git \
		examples/chirpunk/lunex-cyberpunk-assets
	pushd examples/chirpunk/lunex-cyberpunk-assets \
	&& git sparse-checkout set --no-cone assets \
	&& git checkout \
	&& popd
examples/chirpunk/assets: examples/chirpunk/lunex-cyberpunk-assets
	pushd examples/chirpunk \
	&& ../../scripts/x_platform_ln.sh lunex-cyberpunk-assets/assets assets \
	&& popd
examples/chirpunk/assets/menus: examples/chirpunk/assets examples/chirpunk/menus
	pushd examples/chirpunk/lunex-cyberpunk-assets/assets \
	&& ../../../../scripts/x_platform_ln.sh ../../menus menus \
	&& popd

checkout-cyberpunk: examples/chirpunk/assets examples/chirpunk/assets/menus

check:
	cargo clippy --workspace --all-features $(CLIPPY_ARGS)

run:
	# cargo run --bin custom_dsl --features advanced_logging
	RUST_BACKTRACE=1 cargo run --bin custom_dsl --features advanced_logging
	# cargo test -p cuicui_chirp grammar --features 'test_and_doc' # with_method # trace_parser'
	# cargo test -p dsl_and_chirp  --features 'cuicui_chirp/test_and_doc cuicui_chirp/trace_parser'
	# cargo test -p cuicui_dsl  --features test_and_doc
	# RUST_BACKTRACE=0 cargo run --bin chirpunk --features 'cuicui_layout/debug advanced_logging'
	# cd examples/sprite_debug && RUSTC_BOOTSTRAP=1 cargo rustc -p sprite_debug -- -Zunpretty=expanded
	# cd examples/bevypunk && RUSTC_BOOTSTRAP=1 cargo rustc -p bevypunk -- -Z macro-backtrace

pre-hook:
	cargo test --package cuicui_dsl --features test_and_doc
	cargo test --package cuicui_chirp --features test_and_doc
	cargo test --workspace --exclude cuicui_dsl
	cargo test --package dsl_and_chirp --features 'cuicui_chirp/more_unsafe'
	cargo test --package dsl_and_chirp
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
	cargo clippy --workspace $(CLIPPY_ARGS)
	cargo fmt --all -- --check
	cargo clippy --no-default-features --package cuicui_layout_bevy_ui $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout_bevy_sprite $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_layout --features debug $(CLIPPY_ARGS)
	cargo clippy --no-default-features --package cuicui_chirp $(CLIPPY_ARGS)
