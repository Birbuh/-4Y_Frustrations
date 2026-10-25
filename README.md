# -4Y: Frustrations

A desktop 2D space battle simulator built with Rust, Bevy, and Rapier.

Inspired by *X4: Foundations*, the project uses original lore and a 2D presentation. The current focus is a sector battle prototype with AI-controlled fighters, navigation, projectiles, and damage to individual ship modules.

**Status:** early prototype. Combat and movement are under active development; see [Known limitations](#known-limitations) before treating the current behavior as final.

## Getting started

### Requirements

- Rust 1.95 or newer and Cargo, as required by the current Bevy 0.19 dependency.
- A native compiler/linker and the platform development libraries required by Bevy.
- A graphics environment supported by Bevy's renderer.

On Linux, the enabled desktop backends and default Bevy features also require development libraries for windowing, audio, and device input. Build errors from packages such as `alsa-sys`, `libudev-sys`, or `wayland-sys` usually indicate missing system development packages.

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
4. Hold the **right mouse button** and drag to pan; use the **mouse wheel** to zoom.
5. Press **Space** again to pause and place more ships.

AI searches for hostile ships within 2,000 world units. Place opposing ships near each other to trigger combat.

## Controls

| Input | Action | Availability |
| --- | --- | --- |
| Left mouse button | Spawn an allied fighter at the cursor | Paused |
| Middle mouse button | Spawn an enemy fighter at the cursor | Paused |
| Right mouse button + drag | Pan the sector camera | Paused or running |
| Mouse wheel | Zoom the sector camera | Paused or running |
| Space | Toggle the simulation and physics pause | Always |

Manual selection is currently disabled. Although order-assignment code exists, selecting a ship and issuing commands is not an active gameplay control.

## Simulation

### Movement and AI

Ships turn toward their route destinations and move along their local forward direction. The custom movement controller applies acceleration, speed limits, and braking.

Non-player ships evaluate their situation at 0.5-second intervals. They search for nearby hostile ships when they have no route and have not fled. A ship whose weapons are destroyed receives a flee order once.

### Combat and damage

Projectiles damage shields or one of three modules:

| Component | Behavior |
| --- | --- |
| Shields | Absorb hits while their health is positive |
| Core | Destruction removes the ship |
| Engines | Destruction disables movement through the current route-handling logic |
| Weapons | Destruction prevents further firing |

Once shields are depleted, the projectile's position relative to the target determines the module hit: front hits damage the core, rear hits damage engines, and side hits damage weapons. Further hits on destroyed engines or weapons damage the core.

A hit that depletes shields does not transfer excess damage to a module. The next hit is classified using the depleted shield state.

Allied and enemy fighter textures change when engines, weapons, or both are destroyed. Four bars display shields, core health, engine health, and weapon health, in that order.

### Current fighter settings

These values are hardcoded in [`add_ship_from_input`](src/map.rs). They are prototype settings rather than balanced ship definitions.

| Setting | Ally | Enemy |
| --- | ---: | ---: |
| Maximum movement cap | 72 | 75 |
| Acceleration parameter | 23 | 25 |
| Initial shields | 700 | 300 |
| Maximum health per module | 733 | 333 |
| Damage per projectile | 30 | 40 |
| Stored repair setting | 3 | 3 |

Automatic repairs are not implemented. Movement values use the current prototype controller's units; they should not be interpreted as physical units per second.

## Code organization

| File | Responsibility |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Application setup, initial resources, plugins, and system scheduling |
| [`src/map.rs`](src/map.rs) | Map state, camera input, cursor conversion, movement, routes, and orders |
| [`src/ships.rs`](src/ships.rs) | Fighter data, spawning, damage visuals, health bars, projectiles, and collisions |
| [`src/npcs.rs`](src/npcs.rs) | Periodic AI decisions and decision timers |
| `assets/icons/` | Fighter textures and other map artwork |

### Ship entities

A ship is represented by a hierarchy:

```text
Physical ship parent
  Sprite, Transform, Collider, custom Velocity, faction
  |
  +-- Logical icon child
  |     MapIcon containing Fighter state, optional Order and MapRoute
  |
  +-- Health-bar children
        Sprite, Transform, FighterHealthBarFill
```

AI and attack targets refer to the **logical icon child**. Movement and rotation modify the **physical parent**. Collision events identify the parent, so collision handling resolves its icon child to apply damage.

Destroying a ship must remove the parent and its children together. Removing only the icon leaves the sprite and collider behind while making the ship unavailable to targeting.

Sector positions use a positive-Y-up coordinate system. Ship sprites face local +Y. Active cursor conversion uses Bevy's camera projection to map window coordinates into this world.

### System ordering

| Schedule | Work |
| --- | --- |
| `Startup` | Spawn the camera and prototype sector |
| `FixedUpdate` | While unpaused: advance timers, move ships, execute orders, process collision damage, evaluate AI, then reset finished timers |
| `Update` | Camera controls and synchronization, pause input, damage visuals, health bars, and sector drawing |
| `PostUpdate` | After camera updates and transform propagation: convert the cursor position, handle spawning, create bars, and assign manual orders |

The simulation chain lets order execution and AI observe completed timers before they are reset. Cursor-dependent handlers run after coordinate conversion. These dependencies are documented next to the systems in the source.

## Development

Use the committed `Cargo.lock` when building the application:

```bash
cargo check --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets
cargo test --locked
```

The prototype still has compiler warnings, and automated gameplay regression tests are not yet included.

### Generate code documentation

Public types and functions include Rustdoc comments. Include private modules when generating the application documentation:

```bash
cargo doc --locked --no-deps --document-private-items
```

Open `target/doc/y4_frustrations/index.html` to browse the generated documentation. Add `--open` to the command to open it automatically.

### Contributing

Bug reports should include the operating system, reproduction steps, and expected versus observed behavior. For combat issues, include the ship factions and module damage involved.

Keep pull requests focused, describe any gameplay changes, and list the checks performed. Update relevant Rustdoc and this README when controls, ship settings, or system responsibilities change.

## Known limitations

- AI targeting, combat steering, and flight behavior are basic and still being refined.
- All shooters share a firing timer; individual weapon cooldowns are not implemented.
- Ship displacement currently depends on the fixed simulation step, and braking can leave residual motion.
- Manual selection and route drawing are disabled in the application setup.
- Repair settings are stored but do not drive a repair system.
- Missed projectiles have no lifetime limit, and AI timer entries are not removed when ships are destroyed.
- Random flee destinations are not seeded for deterministic replay.

## Roadmap

- Refine targeting, navigation, and combat behavior.
- Improve simulation cleanup and add regression tests.
- Balance fighter statistics and combat.
- Prepare a playable desktop release.

The longer-term vision includes a broader space strategy game; current work focuses on the battle simulator.

## Development logs

Follow progress on the [Stardance project page](https://stardance.hackclub.com/projects/57852).

## License

Licensed under the GNU General Public License v3.0. See [LICENSE.md](LICENSE.md).

This is an independent, unofficial project. It is not affiliated with or endorsed by Egosoft. *X4: Foundations* is referenced as an inspiration.
