# Todo

## Ships
* Sitting in chairs
* Sending signals over the circuit network
* Basic thrusters
* Updating the command block properties and circuits etc when a block is placed
* Speed up bucket fill for planets

## Graphics: 
* Fog

* Ground clutter
* Color clamping?
    * Tried this and didn't like it. Go back?
* Chromatic aberration?
* OPTIMIZE: Combine texture and shadow bind group into one, and lighting and camera. Then lower the bind group limit back to defaults.

## Planets
* HUD debugger

## Collisions
* Fix the collision point. In particular, when multiple points collide, don't choose the middle as the offending point.
* OPTIMIZE: break planets into multiple convex parts. Then only one collision needs to be saved in each convex part; the full tree doesn't need to be explored.

## Player motion
* Fix the player bouncing
* Allow walking around on the planet without sliding