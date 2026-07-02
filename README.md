# explore

A terminal-based exploration and mining simulation in Rust. Generate a procedural world, dispatch scout and miner bots, and harvest resources — all from your terminal.

## Installation

```bash
git clone <repo-url>
cd explore
cargo build --release
```

Requires Rust edition 2024 (1.85+).

## Usage

```bash
cargo run --release
```

### Controls

**Home screen**
| Key | Action |
|-----|--------|
| `g` | Start game |
| `o` | Options |
| `q` | Quit |

**Options screen**
| Key | Action |
|-----|--------|
| `↑`/`↓` `k`/`j` | Navigate options |
| `←`/`h` | Decrease value |
| `→`/`l` | Increase value |
| `0-9` | Enter seed digits |
| `⌫` | Delete seed digit / go home |
| `Enter` | Start game |
| `Esc` | Home |

**Game screen**
| Key | Action |
|-----|--------|
| `Space` | Pause/resume |
| `→`/`l` | Step 1s (paused) |
| `h` | Home |
| `r` | Restart |
| `1` | Focus map |
| `2` | Focus minerals |
| `↑`/`↓` `k`/`j` | Scroll minerals |

### Options

- Energy / diamond count (1–40)
- Terrain detail (octaves 1–6) and feature scale (frequency)
- Seed for deterministic worlds
- Scout count (1–8) and algorithm: `FrontierWavefront`, `AStarExploration`, `RandomWalkCostBias`, `BFS`, `DFS`
- Miner count (1–8) and algorithm: `A*`, `Dijkstra`, `Bidirectional`
- Assignment strategy: `LeastAssigned`, `RoundRobin`, `WeightedByValue`

## Architecture

```
main.rs → app.rs → state/ (screen, store, game_world, clock)
                  → bots/ (manager, scout, miner, movement, pathfinding, types)
                  → map.rs
                  → event.rs
```

- **`App`** owns `State`, runs the terminal loop, and dispatches rendering.
- **`State`** is a reducer-like store managing screens, options, and the optional `GameWorld`.
- **`GameWorld`** owns the simulation: `Map`, `GameClock`, `BotManager`, known minerals, and resources.
- **`Map`** is procedurally generated with Perlin/Fbm noise → elevation bands → terrain. Minerals are placed deterministically from a seed. Shared across bot threads via `Arc<Map>`.
- **`BotManager`** spawns scout and miner threads. Communication is channel-based (`mpsc`): ticks flow in, `BotEvent`s flow out.
- **Scouts** explore using configurable algorithms (BFS, DFS, frontier wavefront, A* exploration, random walk). Discovered minerals are reported only after returning to base.
- **Miners** receive `MiningOrder`s, path to the mineral (A*, Dijkstra, or bidirectional BFS), mine one unit, and return to base.
- **`GameClock`** is a fixed-step (≈30 Hz) accumulator clock with pause and manual step support.
- **Rendering** uses `ratatui` with terrain colors, bot symbols (`S`/`M`), mineral panels, and status bars.

### Key Design Choices

| Choice | Why |
|--------|-----|
| `Arc<Map>` | Read-only shared map across bot threads, no locks needed |
| `mpsc` channels | Bot threads never mutate `GameWorld` directly — events are drained on the main thread |
| `Option<GameWorld>` | No active world on home/options screens |
| Fixed-step clock | Simulation speed decoupled from render speed |
| Scout returns to base | Models delayed information: minerals are "known" only after the scout delivers the report |
| Algorithm traits | Map generation and pathfinding are extensible via traits/enums |

## Tests

```bash
cargo test
```

Coverage includes: state transitions, deterministic map generation, movement timing, pathfinding, exploration bias, and UI snapshot testing (via `insta`).

## Built With

- [`ratatui`](https://github.com/ratatui-org/ratatui) — terminal UI
- [`crossterm`](https://github.com/crossterm-rs/crossterm) — terminal input
- [`noise`](https://github.com/Razaekel/noise-rs) — Perlin/Fbm terrain generation
- [`pathfinding`](https://github.com/samueltardieu/pathfinding) — graph search algorithms
- [`figlet-rs`](https://github.com/edsrzf/figlet-rs) — ASCII titles
- [`insta`](https://github.com/mitsuhiko/insta) — snapshot testing

## Authors

Jules B., Younes E., Mathias D.
