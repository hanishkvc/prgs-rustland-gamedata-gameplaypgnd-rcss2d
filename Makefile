
cbuild:
	cargo build
	cargo build --release

cbuild-inbtw:
	cargo build --features inbetween_frames
	cargo build --release --features inbetween_frames

cclean:
	cargo clean

crun:
	cargo run --release

