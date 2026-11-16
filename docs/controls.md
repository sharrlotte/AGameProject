# Controls

## Game scene

| Input | Action |
|---|---|
| `W` / `↑` | forward thrust (skip AI control of controlled ship) |
| `S` / `↓` | reverse thrust |
| `A` / `←` | torque left |
| `D` / `→` | torque right |
| `Q` | strafe left |
| `E` | strafe right |
| `Shift` (L) | camera pan speed ×3 |
| Mouse wheel | zoom (`MIN_ZOOM 0.2` .. `MAX_ZOOM 30.0`) |
| `Ctrl` + `Left` drag | box-select player entities (ships listed in `EntityType`) — release fires selection |
| `Escape` | toggle Pause (playing/paused) |
| Ship list buttons (HUD) | 1st click follow, 2nd click control, 3rd clear |

## Ship Editor

| Input | Action |
|---|---|
| Left click (release, no drag) | place block (face-connected) |
| Left+drag | pan camera |
| Right click (hold) | delete block |
| Mouse wheel | zoom `[0.1, 5.0]` |
| `Ctrl` + wheel | pan horizontally instead of zoom |
| `Back to Home` | `SceneState::Home` |
| `Deploy` | set `current_ship` (consumed by next mission battle as the player's ship) |
| `Start Game` | `SceneState::Game` |
| `Back` button | `SceneState::Home` |

## Pause menu

| Input | Action |
|---|---|
| `Escape` | resume |
| Resume / Settings / Save / Load / Home / Exit | buttons |

## Turret control (keyboard)

None — `TurretControl` has been removed; player turrets auto-target like AI turrets.

## Battle end screen

| Input | Action |
|---|---|
| Retry | full battle reset (re-enters Home → Game) |
| Home | return to Home; battle stays over until re-entry |
