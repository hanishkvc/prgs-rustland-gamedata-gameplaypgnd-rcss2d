####################
Playback Playground
####################

Author: HanishKVC
Version: 20221118IST2325
License: GPL-3.0+


Overview
############

Allow controlled playback and look at captued game data like from robocup
soccer simulator for example. Also allow connecting to a live server to get
and display the game data.

In the long run additionally allow augumenting of displayed player movements
playback additionally with info captured manually or automatically (during
the game or later). This can include

* game actions captured like goal, kick/tackle/... and good/bad pass/...,
  penalty, cards, ...

* color coding of

  * performance from captured game actions.

  * coarse grained view of captured/tracked health params like stamina, ...

* overlapping of past games data wrt movements/actions/performance in useful
  ways.


Usage
#######

Cmdline arg
============

Rcg Playback
--------------

Pass the rcss rcg file as the 1st and only argument to the program.
This will playback the contents of the rcg file.

Live
------

By passing live as the 1st argument to the program, one can make the program
work as a simple and currently very minimal robocup soccer sim monitor.
It can be used to watch a game live as well as kick-start(kick-off) wrt the
2 halfs+ if & when needed.

One can pass a 2nd argument to the program, following live, and it will be
used as the address of the robocup server to connect to. Else it will try
to connect to the server on port 6000 on the local machine.

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

* Internal debug cmds

  * d -> to dump current data associated with entities in the playground

    * ie players, ball, msgs

Msgs
=====

One can see the following messages on the screen in addition to the
player movements.

* the score, at the top left

* the game time as represented by the playdata source, at the top mid

* game related messages in the play data, at the bottom left.

* any unknown/unhandled messages in the play data, at the bottom mid

* the set and actual fps, at the top right

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

