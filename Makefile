
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
	target/release/gameplaypgnd-rcss2d --mode rclive

test1:
	target/release/gameplaypgnd-rcss2d --mode rcg --src gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.rcg

test1_vb:
	target/release/gameplaypgnd-rcss2d --mode rcg --src gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.rcg --virtball gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.virtball.csv

test1_vb_save:
	target/release/gameplaypgnd-rcss2d --mode rcg --src gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.rcg --virtball gamedata/20210626230154-ThunderLeague_21-vs-Hades2D_0.virtball.csv --save_interval 1

test2:
	target/release/gameplaypgnd-rcss2d --mode rcg --src gamedata/20221118233608-tm01_3-vs-tm02_1.rcg

test2_vb:
	target/release/gameplaypgnd-rcss2d --mode rcg --src gamedata/20221118233608-tm01_3-vs-tm02_1.rcg --virtball gamedata/20221118233608-tm01_3-vs-tm02_1.virtball.csv

rgb2png:
	for i in /tmp/gppgnd*rgb; do echo $$i; gm convert -size 1024x600 -depth 8 -format rgb $$i $$i.png; done

png2mp4:
	mencoder mf:///tmp/gppgnd*png -mf fps=10 -o /tmp/gppgnd.mp4 -ovc lavc

