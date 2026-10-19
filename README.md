# -4Y: Frustrations

**A 2D space battle simulator built with Rust and Bevy.**

-4Y: Frustrations is an open-source, non-commercial space-combat project inspired by *X4: Foundations*. For now, development is focused on a smaller, achievable goal: a battle simulator featuring ship navigation, rotation, AI-controlled opponents and modular ship damage.

The project uses original lore and a 2D presentation rather than 3D graphics.

> **Development status:** Early prototype. Core movement and combat mechanics are being developed, and many planned features are not yet implemented.

## Features

### Ship movement and navigation

- Ships accelerate and decelerate instead of moving at a constant speed.
- Ships rotate to face their destinations.
- Ships can rotate while moving slowly.
- Movement orders generate visible routes on the sector map.
- Ships use their orientation to determine their direction of travel.

### Combat

Ships can attack other ships and damage individual modules.

The current damage system includes four components:

- **Shields:** Absorb damage before the other modules are targeted.
- **Weapons:** Losing this module prevents a ship from attacking.
- **Engines:** Losing this module prevents a ship from moving.
- **Core:** Destroying the core destroys the ship.

After the shields are depleted, the attacker's position relative to the target determines which module is hit. Attacks from the sides target weapons, attacks from behind target engines, and attacks from the front can hit the core.

If a module has already been destroyed, subsequent damage can reach the core instead.

### AI-controlled ships

Non-player ships periodically search for nearby hostile ships and can assign themselves attack orders.

Combat movement and targeting are still under development. The AI is currently a basic prototype rather than a complete tactical system.

### Manual ship spawning

During the paused state, ships can be spawned directly at the cursor position:

| Input | Action |
|---|---|
| Left mouse button | Spawn an allied fighter |
| Middle mouse button | Spawn an enemy fighter |
| Space | Toggle the simulation between paused and unpaused |

Both ship types currently use fixed prototype statistics:

| Property | Value |
|---|---:|
| Maximum speed | 10 |
| Acceleration | 3.3 |
| Shields | 3,000 |
| Maximum module health | 3,333 |
| Repair speed | 123 |
| Attack damage | 20 |

The allied and enemy fighters use different sprites and faction relationships. These values are currently hardcoded for testing, not balanced gameplay.

## Technology

- **Language:** Rust
- **Game engine:** Bevy 0.19
- **UI:** bevy_egui
- **Input management:** leafwing-input-manager
- **Graph algorithms:** petgraph
- **Serialization:** Serde and RON

The project uses Bevy's entity-component-system architecture to organise ships, movement, combat and simulation systems.

## Getting started

### Requirements

- Rust and Cargo
- A graphics environment supported by Bevy 0.19
- The platform dependencies required by Bevy

### Run from source

Clone the repository:

```bash
git clone https://github.com/Birbuh/-4Y_Frustrations.git
cd -4Y_Frustrations
cargo run
```

For an optimised build:

```bash
cargo build --release
```

The commands above assume the current source compiles successfully on your platform.

## Development roadmap

The immediate goal is to develop the battle simulator into a coherent, playable experience.

Next steps will include:

- Making module damage visible to the player.
- Balancing ship statistics and combat.
- Publishing a playable release on itch.io

The original long-term vision includes a broader space strategy game, but the current focus is deliberately narrower.

## Development logs

Follow progress on the [Stardance project page](https://stardance.hackclub.com/projects/57852).

## License

Licensed under the GNU General Public License v3.0. See [LICENSE.md](LICENSE.md) for details.

## Disclaimer

-4Y: Frustrations is an independent, unofficial project. It is not affiliated with, endorsed by, or sponsored by Egosoft. *X4: Foundations* is referenced as an inspiration; the project does not claim ownership of Egosoft's intellectual property.
