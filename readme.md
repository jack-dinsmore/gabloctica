# Todo

## Graphics: 
* Sky box
* Fog
* Color clamping?

## Planets
* HUD debugger

## Collisions
* Fix the collision point. In particular, when multiple points collide, don't choose the middle as the offending point.
* OPTIMIZE: break planets into multiple convex parts. Then only one collision needs to be saved in each convex part; the full tree doesn't need to be explored.

## Player motion
* Fix the player bouncing
* Allow walking around on the planet without sliding