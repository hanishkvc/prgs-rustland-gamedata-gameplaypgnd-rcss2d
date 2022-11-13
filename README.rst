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

Pass the rcss rcg file as the 1st and only argument to the program.
This will playback the contents of the rcg file.

One can use the following keys to control the behaviour as noted below.

* p -> to pause/unpause the playback

* b -> to hide/unhide the ball

* Seeking

  * right arrow key -> to seek/jump forward

  * left arrow key -> to seek/jump backward

* FPS - frames per second

  * f -> to reduce the current fps

  * F -> to increase the current fps

* c -> to change the background color

* Internal debug cmds

  * d -> to dump current data associated with entities in the playground

    * ie players, ball, msgs

