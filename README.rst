####################
Playback Playground
####################

Author: HanishKVC
Version: 20221111IST1745
License: GPL-3.0+


Overview
############

Allow controlled playback and look at captued game data like from robocup
soccer simulator for example.

In the long run additionally allow augumenting of displayed player movements
playback additionally with info captured manually or automatically (during
the game or later). This can include

* game actions captured like goal, good or bad pass, penalty, cards, ...

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

By passing live as the 1st and only argument to the program, one can make the
program work as a simple and currently very minimal robocup soccer sim monitor.
It can be used to watch a game live as well as kick-start(kick-off) wrt the
2 halfs+ if & when needed.


Keys
======

One can use the following keys to control the behaviour as noted below.

* p -> to pause/unpause the playback

* h -> to hide/unhide the help msg box

* b -> to hide/unhide the ball

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

* c -> to change the background color

* 1 -> If working in the RobocupSoccerSim monitor mode, this allows to send
  the kick-off (dispstart) command to the server.

* Internal debug cmds

  * d -> to dump current data associated with entities in the playground

    * ie players, ball, msgs

Msgs
=====

One can see the following messages on the screen in addition to the
player movements.

* the score, at the top left

* the game time as represented by the playdata source, at the top right

* game related messages in the play data, at the bottom left.

* any unknown/unhandled messages in the play data, at the bottom mid

* the set and actual fps, at the top mid

