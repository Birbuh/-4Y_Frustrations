# -4Y: Frustrations

A small 2D space battle simulator built with Rust, Bevy, and Rapier.

**Status:** early prototype. I don't have time to actively work on it.

I also tried to make a web version on itch.io but that would require too much rewriting...

## Getting started

You can download from GitHub releases or build from source:

### Requirements

Just install Rust 1.95 or newer from Rustup.
If you get errors involving alsa-sys, libudev-sys, or wayland-sys, you're probably missing some system dependencies.

### Run from source

```bash
git clone https://github.com/Birbuh/-4Y_Frustrations.git
cd -4Y_Frustrations
cargo run --locked
```

For an optimized build:

```bash
cargo run --release --locked
```

Run these commands from the repository root so Bevy can find the `assets/` directory. The application opens a fullscreen sector view and starts paused with no ships.

### Start a battle

1. Left-click in the sector to place allied fighters.
2. Middle-click elsewhere nearby to place enemy fighters.
3. Press **Space** to start the simulation.

Place opposing ships near each other to trigger combat.
You can pause always with **Space** and place more ships! 

## Controls
### ONLY IN PAUSE:
Left mouse button -> Spawn an allied fighter at the cursor

Middle mouse button -> Spawn an enemy fighter at the cursor
### ALWAYS ENABLED:
Right mouse button + drag -> Pan the sector camera

Mouse wheel -> Zoom the sector camera

Space -> Toggle the simulation and physics pause

There is a selection system present in the code, but it's not used. Why? Because you can't give orders to Allies/Enemies, it would be senseless to have it enabled.

## Simulation

### Movement and AI

Ships turn toward their route destinations and move along their local forward direction. The custom movement controller applies acceleration, speed limits, and braking.

Every 0.5 seconds, ships without a route look for nearby enemies, unless they're already fleeing or fleed.

### Combat and damage

Projectiles damage shields or one of three modules:

Shields -> Absorb hits while their health is positive;

Core -> Destruction removes the ship;

Engines -> Destruction disables movement through the current route-handling logic;

Weapons -> Destruction prevents further firing. The ship tries to escape, but only once.

Once shields are depleted, the projectile's position relative to the target determines the module hit: front hits damage the core, rear hits damage engines, and side hits damage weapons. Further hits on destroyed engines or weapons damage the core.

Four bars display shields, core health, engine health, and weapon health in this exact order from up to down.

### Current fighter settings

These values are hardcoded in [`add_ship_from_input`](src/map.rs). I tried to make Allies more focused on durability, and Enemies on attack. These will probably change in the future.

| Setting | Ally | Enemy |
| --- | ---: | ---: |
| Maximum movement cap | 72 | 75 |
| Acceleration parameter | 23 | 25 |
| Initial shields | 700 | 300 |
| Maximum health per module | 733 | 333 |
| Damage per projectile | 30 | 40 |
| Stored repair setting | 3 | 3 |

Automatic repairs are not implemented yet. Movement values should not be interpreted as physical units per second.

## License

Licensed under the GNU General Public License v3.0. See [LICENSE.md](LICENSE.md).

This is an independent, unofficial project. It is not affiliated with or endorsed by Egosoft. *X4: Foundations* is referenced as an inspiration.
