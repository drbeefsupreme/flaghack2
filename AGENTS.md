# Flaghack2 Agent Brief

This repo is a Rust/Macroquad prototype of **Flaghack2**, a 2D roguelike with smooth movement and a stylized top-down/isometric-adjacent look. The current focus is building a playable sandbox with procedural vector-style sprites and a large tiled map.

## Project Summary
- **Engine:** Rust + Macroquad
- **Style:** Flat, Rimworld-inspired silhouettes, bold color blocks, simple shapes
- **Core loop (current):** WASD movement, mouse flag placement/pickup, ley lines between nearby flags, animated flags, large background map with camera pan/zoom
- **Scale:** The map is huge; all in-world models are scaled down to `MODEL_SCALE = 0.25` and the camera defaults to a zoom of `4.0` so the map appears ~4x larger without enlarging tiles.

## Key Modules
- `src/main.rs`
  - Game state machine: `Title` → `ClassSelect` → `Dungeon`
  - Input handling, camera logic, HUD
  - Now uses `src/constants.rs` for shared constants
- `src/camps.rs`
  - Camp definitions (polygons, colors, notice text)
  - Centralized spawn lists for tents/chairs/campfires/flags/hippies per camp
  - Current camps: t3mpcamp, Georgia Peanuts, DeBussy Bus Station
- `src/hud.rs`
  - HUD rendering (flags, speed, total flags, player coordinates)
- `src/flag_state.rs`
  - Centralized flag inventory/transfer logic (ground/player/hippies) + total invariant tracking
- `src/map.rs`
  - Loads tiled PNGs from `assets/map/tiles/`
  - Draws only visible tiles
  - Map dimensions + travel speed helpers
  - `MapRegion::contains_point` for camp entry checks
- `src/camera.rs`
  - Camera state (zoom + pan + drag)
  - `DEFAULT_ZOOM = 4.0`
  - `flip_zoom_y` fixes upside-down map
- `src/player.rs`
  - Procedural “Vexillomancer” sprite with facing logic
  - Split robe (yellow/black), head front/back, hands per facing
- `src/flags.rs`
  - Flag data + placement/pickup logic
  - Wiggle animation, wind support
  - `make_flag` helper for spawning dropped flags with phase
- `src/geom.rs`
  - Geometry helpers (polygon bounds, point-in-polygon, line sampling)
- `src/ley_lines.rs`
  - Ley line geometry between nearby flags
  - Pentagram detection (5-flag ring) marks lines as `Pentagram`
- `src/scenery.rs`
  - Procedural tents, chairs, campfires, trees, geodesic domes
  - Domes can contain decorations; first decoration is a rotating crystal
  - Crystal dome fixed at `(4900, 3184)` and large campfire at `T3MPCAMP_CAMPFIRE_POS`
- `src/npc.rs`
  - Hippie NPCs (stick figure) with facing, wandering inside polygon region
  - Spawned via camp spawn lists
  - Hippies can carry up to 2 flags, pick up nearby flags, and drop them on a timer
  - Stealing flags makes them angry: red/orange cycling head glow, chase at ~66% player speed
  - Angry hippies steal flags back, then flee for 10 seconds
  - 1s anger delay before they can steal back
  - Angry hippies ignore camp boundaries; calm hippies return to camp
  - Hippies keep a collision bubble so they do not overlap
  - If the player has no flags, hippies stop chasing
  - Random initial flags per hippie (33% chance of 1, 10% chance of 2)
- `src/scale.rs`
  - `MODEL_SCALE = 0.25` + helper `scaled()`
- `src/constants.rs`
  - Centralized gameplay constants (flags, ley lines, pentagram FX, map, t3mpcamp, hippie behavior)

## Assets
- **Title screen mark:** `assets/png/signifiersmark.png`
- **Map tiles:** `assets/map/tiles/tile_X_Y.png` (1024x1024 each)

## Map Pipeline (PDF → tiles)
Source map: `/home/drbeefsupreme/Alchemy Legends Map_3by5_Final.pdf`

Commands used (150 DPI, padded to 9216x6144, then 1024x1024 tiles):
```
convert -density 150 ".../Alchemy Legends Map_3by5_Final.pdf" -background white -alpha remove -alpha off /tmp/alchemy_map.png
convert /tmp/alchemy_map.png -background white -gravity northwest -extent 9216x6144 /tmp/alchemy_map_padded.png
convert /tmp/alchemy_map_padded.png -crop 1024x1024 +repage /tmp/map_tiles/tile_%04d.png
# rename sequential tiles to assets/map/tiles/tile_x_y.png (9 cols x 6 rows)
```

## Gameplay Notes
- Movement is smooth (not tile-based).
- Flags can be placed near the player (LMB) and picked up (RMB).
- Player starts with `10` flags in inventory.
- Ley lines glow between nearby flags; brightness scales with distance.
- Ley lines cap at `150` units; pentagram formations glow red/orange.
- Pentagram detection has relaxed radius/angle tolerance for larger shapes.
- Standing in a pentagram center spawns rainbow sparkles that persist until they fade out at max radius.
- Camera pans with middle mouse drag; zooms with mouse wheel.
- HUD includes flags count, speed, total flags, and player coordinates (bottom-right).
- Entering a camp shows its banner centered for 4 seconds with 0.5s fade in/out. Names switch instantly when crossing between camps.
- Current camps: t3mpcamp (`t3mpcamp.com`), Georgia Peanuts, DeBussy Bus Station.
- Hippies drop one flag with 25% chance every 30s and ignore pickups for 30s after a drop.
- Total flags in the game are treated as an invariant (debug assert + HUD total).

## Testing
We are doing TDD. Every feature gets at least one unit test.
Run:
```
cargo test --release
```

## Conventions / Expectations
- Keep new features test-covered.
- Preserve the procedural vector-style art approach unless explicitly asked to add raster assets.
- Keep world assets scaled by `MODEL_SCALE`, not by shrinking tiles.
- Avoid rasterizing SVGs at runtime; pre-rasterize and commit assets.

## Known Intentions
- This map is temporary (guide layer); eventually we’ll overlay a custom map.
- Domes will support multiple decoration types.
- Ley line geometry will become a core gameplay system.

## Recent Additions (Session Summary)
- `src/constants.rs` created; constants removed from `src/main.rs`.
- Ley lines now shimmer purple/pink normally, shift to red/orange for pentagrams.
- Pentagram centers tracked for sparkle effects.
- Sparkle system is time-based spawn; particles travel outward, fade to 0 at max radius, and persist after leaving the pentagram.
- Camps system added in `src/camps.rs` (t3mpcamp, Georgia Peanuts, DeBussy Bus Station) with spawn lists for scenery, flags, and hippies.
- Camp banner system now uses camp names and switches immediately between overlapping camps.
- Hippie NPCs added with simple stick-figure model and bounded wandering; angry hippies can chase outside camp and calm hippies return.
- Hippies can carry/pick up flags, steal from the player (RMB), get angry, chase, steal back, and flee; they stop chasing if the player has no flags.
- Hippies can drop flags on a timer and temporarily ignore pickups; they also maintain a small collision bubble.
- HUD moved into `src/hud.rs` and shows total flag count.
