
cbuild:
	cargo build
	cargo build --release

cbuild-inbtw:
	cargo build --release --features inbetween_frames

cclean:
	cargo clean

crun:
	cargo run --release

