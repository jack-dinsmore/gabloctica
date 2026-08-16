# Todo

## Graphics: 
* Billboards and ground clutter
* Fix the ship collision thing
    - I think the offset is pulling it into the ground, hence the problem
* Static friction
* Fog

## Biscuit
* Finish machine implementation
* Emulator
* Return
* Automatically drop tables whenever they go out of scope
* Don't duplicate literals

## Ships
* Updating the command block properties and circuits etc when a block is placed
* Speed up bucket fill for planets

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