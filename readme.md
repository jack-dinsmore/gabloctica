# Todo

## Graphics: 
* More terrain generation
    * Make the planet mass accurate even when only part of the planet is loaded
* Chromatic aberration?
* OPTIMIZE: Combine texture and shadow bind group into one, and lighting and camera. Then lower the bind group limit back to defaults.

## Player motion
* Revisit friction
* Prevent ship from popping up
* Make some kind of torque which resists spinning

## Biscuit
* Finish machine implementation

## Ships
* Speed up bucket fill for planets

## Collisions
* Fix the collision point. In particular, when multiple points collide, don't choose the middle as the offending point.
* OPTIMIZE: break planets into multiple convex parts. Then only one collision needs to be saved in each convex part; the full tree doesn't need to be explored.

## TITLE IDEAS
* Tetronomy