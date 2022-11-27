####################
Playback Playground
####################

Author: HanishKVC
Version: 20221125IST1911
License: GPL-3.0+


Overview
############

Allow controlled playback and look at captued game data like from robocup
soccer simulator for example. Also allow connecting to a live server to get
and display the game data.

Additionally allow augumenting of displayed player movements playback with
info captured manually or automatically (during the game or later). This
can include

* game actions captured like goal, kick/tackle/... and good/bad pass/...,
  penalty, cards, ...

  * try infer good/bad passes (minimal logic for now), good/self goal,
    also performance scoring based on infered/otherwise game actions

* color coding ++ of

  * performance from captured game actions.

    * if reqd in future. Currently shown has a relative bar graph,
      if requested.

  * coarse grained view of captured/tracked health params like stamina, ...

* overlapping of past games data wrt movements/actions/performance in useful
  ways.


Usage
#######

Cmdline arg
============

Rcg Playback
--------------

--mode rcg --src <path/file>

Set the mode and specify the rcss rcg file as mentioned above.
This will playback the contents of the rcg file.

RC Live
--------

--mode rclive [--src <nw address>]

This runs the program as a simple and minimal robocup soccer sim monitor.
It can be used to watch a game live as well as kick-start(kick-off) wrt the
2 halfs+ if & when needed.

If the --src argument is specified, it will be used as the address of the
robocup server to connect to. Else it will try to connect to the server on
port 6000 on the local machine.

NOTE: Ideally one needs to start the rc server first, before starting this
program, this will ensure tha tthe initial init handshake that is sent when
this program is started, to the server, will succeed. However if one starts
the rc server after this program, then one can use key 0 to initiate the
initial handshake.

NOTE: Also do note that when ever the init handshake is successful, and the
server sends out a message to this program, it switchs the server address to
point to the address(including port) from which the message was recieved.
From then on this program cant connect to a freshly/newly started server, as
the internally stored server address has changed. So If one wants to connect
to new rc server again, after a previous successful handshake + msg, one needs
to quit this program and start it fresh again.

Saving playback frames
-----------------------

--save_interval <OnceEvery???Frames>

Fps
------

--fps <The.Fps>

Current flow, overrides this with the fps suggested by playdata source.

Virtual ball
--------------

--virtball <path/virtball.csv>

VirtBall.CSV should be a csv file containing a series of records, consisting
of time stamp/counter, ball x, ball y. Inturn the logic will show a virtual
ball, by interpolating where required.


Keys
======

One can use the following keys to control the behaviour as noted below.

* p -> to pause/unpause the playback

* h -> to hide/unhide the help msg box

* s -> enter set-show/hide-mode

  * s -> to show/hide the display of stamina

  * a -> to show/hide the display of actions

  * b -> to show/hide the ball

  * any other key -> exit set-show/hide-mode

* Seeking

  * right arrow key -> to seek/jump forward

    * NOTE: Any messages in the skipped records, wont be shown.

  * left arrow key -> to seek/jump backward

    * NOTE: The messages shown dont get reverted back wrt time.

    * NOTE: Seeking back after reaching end, will bring back the source
      to be alive.

* FPS - frames per second

  * f -> to reduce the current fps

  * F -> to increase the current fps

  NOTE: This helps change the rate of playback in the default one record per
  frame mode. However in the interpolated movement mode, changing fps, doesnt
  allow one to change the rate of playback.

* b -> to change the background color

* c -> enter send-record-coded-mode

  If working in RobocupSoccerSim monitor live mode, then

  * 0 -> send the initial handshake (dispinit) command to the server

  * 1 -> send the kick-off (dispstart) command to the server.

  * any other key -> exit send-record-coded-mode

* d -> enters internal-debug-mode

  * e -> to dump current data associated with entities in the playground

    * ie players, ball, msgs, actions info, ...

  * a -> to show ActionsInfo relative perf summary based on best team
    local performance.

  * A -> to show ActionsInfo relative perf summary based on best perf
    across both teams.

  * NOTE: Pressing <a> when already in <a> mode, clears it. Same with <A>.
    However pressing <a> when in <A> or otherway, changes the summary type.

  * d -> to show ActionsInfo relative distance traversed summary based on
    most distance traversed wrt own team players.

  * D -> to show ActionsInfo relative distance traversed summary based on
    most distance traversed across both teams.

  * any other key -> exit internal-debug-mode


Msgs
=====

One can see the following messages on the screen in addition to the
player movements.

* the score, at the top left

* the game time as represented by the playdata source, at the top mid

* game related messages in the play data, at the bottom left.

* any unknown/unhandled messages in the play data, at the bottom mid

* curently active starting key in multikey cmds and set+actual fps,
  at the top right

Augumenting
=============

The following geometric characteristics wrt the player could be used
to map to different player performance and or other characteristics

* color and its shading of the player

* colors of the 4 outerlines around the player, Currently
  * both vertical lines are mapped to player stamina by default
    good stamina is green, in between is yellow and low is red
  * both (top and bottom) horizontal lines are mapped to any card
    issued to player, for now.

* color and arc length of the arc around the player
  Currently it is mapped to actions like kick, tackle, catch


Notes
#######

Scoring wrt Bad pass
======================

During a pass, if the recieving player foolishly or due to lack of experience
/skills, fails to take the pass, currently the logic will only penalise the
sender of the pass and not the failed reciever. Which in a way may be fine,
in real world as the sender should know whether the receiver is capable or
not, in a way to an extent !?! However wrt current robocup teams, I may have
to look at position of ball and players and inturn penalise really nearby
players, during a failed/bad pass to some extent ???

Virtual Ball
================

If there is no ball information along with game data, use game actions like
kick, tackle, catch, etal to interpolate a virtual ball.

Use a two pass flow, where 1st capture the useful ball related game actions
and inturn use it to visualise a virtual ball using interpolation.

When the playdata source indicates that the playback has reached the end, the
logic will automatically capture the required actions related data, into a
tmp file.

NOTE: The logic accounts for seeking in a crude way, currently, which should
be ok to an extent.


Changelog
###########

Look at git log in general, the below captures things only sometimes.

20221123++
============

Patched the latest external release wrt below and inturn rebased the currently
internal exploration on top of the same

* fixing Rcg helper to support non hex state info and stamina record at almost
  any position within the player record.

* add support for opting out of WM_PING mechanism in sdl helper

* consume all events before handling the playback and related logic

Infer passes and their success or failure and inturn score the same. Also track
the distance moved/traversed by players. Allow comparing these wrt best in same
team as well as across both teams.

Add support for tagged commandline arguments.

Virtual ball, if required.

Infer goal as a good or a self goal and identify the player responsible for same

Timed messages box for user config change at runtime

20221126++
============

Update the handle action logic to check thro all possible prev and cur action
sequence possibilities, to a greater extent, with the new flow, in a explicit
manner.

Account for -ve scores, wrt the relative perf bars based graphical score summary
logic.

Determine program window resolution dynamically at runtime based on screen res.

