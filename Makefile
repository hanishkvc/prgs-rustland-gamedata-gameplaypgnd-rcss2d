
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
	target/release/playbackpgnd --mode rclive

test1:
	target/release/playbackpgnd --mode rcg --src gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.rcg

test1_vb:
	target/release/playbackpgnd --mode rcg --src gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.rcg --virtball gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.virtball.csv

test2:
	target/release/playbackpgnd --mode rcg --src gamedata/20221118233608-tm01_3-vs-tm02_1.rcg

rgb2png:
	gm convert -size 1024x600 -depth 8 -format rgb /tmp/ppgnd010.rgb /tmp/test.png
