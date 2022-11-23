
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

rclive:
	target/release/playbackpgnd live

test1:
	target/release/playbackpgnd gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.rcg

test2:
	target/release/playbackpgnd gamedata/20221118233608-tm01_3-vs-tm02_1.rcg

