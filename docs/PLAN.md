# Flaghack MVP Plan (Notcurses + TDD)

## Summary
Build a minimal Nethack‑like loop in Python with Notcurses: a title screen, a single‑class selection screen, and a small fixed dungeon with 8‑direction movement. All tests must initialize and render with real Notcurses and require a real TTY.

## Scope (MVP)
- Title screen with game name and “Press any key”.
- Class selection with a single class.
- Fixed small dungeon map.
- Player `@`, walls `#`, floors `.`.
- Turn‑based movement with vi keys (`h j k l y u b n`) and arrow keys.
- `q` quits from any screen.

## Architecture
- Scene state machine: `TITLE → CLASS_SELECT → DUNGEON`.
- Core logic is pure Python; rendering always uses Notcurses.
- No fake renderer; tests must import Notcurses and render once per test case.

## Public Interfaces
- `Action` enum: `MOVE_N`, `MOVE_S`, `MOVE_E`, `MOVE_W`, `MOVE_NE`, `MOVE_NW`, `MOVE_SE`, `MOVE_SW`, `CONFIRM`, `BACK`, `QUIT`, `NOOP`.
- `Scene` protocol: `handle(action) -> SceneTransition`, `render(nc: Notcurses) -> None`.
- `SceneTransition`: `next_scene: Scene | None`, `quit: bool`.
- `GameState` models: `Player(pos)`, `Map(tiles, width, height)`, `ClassChoice(name)`.

## Project Layout
- `flaghack/pyproject.toml`
- `flaghack/flaghack/app.py` (Notcurses init + main loop)
- `flaghack/flaghack/input.py` (Notcurses input → Action)
- `flaghack/flaghack/scenes/title.py`
- `flaghack/flaghack/scenes/class_select.py`
- `flaghack/flaghack/scenes/dungeon.py`
- `flaghack/flaghack/models.py`
- `flaghack/assets/maps/tutorial_1.txt`
- `flaghack/tests/…`

## Notcurses Test Policy
- Tests must create a real `Notcurses()` context.
- Require real TTY; if `sys.stdin.isatty()` or `sys.stdout.isatty()` is false, fail fast with a clear error.
- Rendering assertions are “smoke only”: call `render()` and assert no exception.
- Graphics validation (Kitty/Sixel/pixel blitting) will be run locally by the developer.

## Tests
- `test_title_scene.py`
  - Renders title and prompt without exception.
  - Any key triggers transition to class select.
- `test_class_select_scene.py`
  - Renders single class without exception.
  - `CONFIRM` transitions to dungeon.
  - `BACK` returns to title.
- `test_map_load.py`
  - ASCII map parses to expected size.
  - Player spawn is on floor.
- `test_movement.py`
  - Movement works for all vi directions.
  - Arrow keys map to cardinal directions.
  - Walls and boundaries block movement.
- `test_dungeon_render.py`
  - Render call succeeds for a loaded map and player position.

## Assumptions
- Single class name for MVP: `Vexillomancer`.
- Fixed map size ~20x12 for MVP.
- No combat, monsters, inventory, or FOV yet.
