# Camera Spec

## Design frame

- `VIEW_HEIGHT` (`constant.rs`) is `1080.0` world units: the game is authored for a
  1920×1080 display at `ortho.scale = 1.0`, where one world unit maps to one pixel.
- The primary `Camera2d` uses
  `Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: VIEW_HEIGHT }, ..OrthographicProjection::default_2d() })`.
- Visible world height is always `VIEW_HEIGHT * ortho.scale`; visible world width is
  `height * window_aspect`. Resizing, maximizing, restoring, DPI changes, and fullscreen
  transitions change pixel density (and width for non-16:9 aspects) but never the visible
  world height.
- `ScalingMode::FixedVertical` is chosen over `WindowSize` (the `Camera2d` default, which
  reveals more world as the window grows), `AutoMax` (zooms in on ultrawide), and `Fixed`
  (stretches on non-16:9).
- One `Camera2d` is spawned at `Startup` and persists across states; Game and Ship Editor
  share the projection. There is no per-scene projection switching.

| Viewport | Aspect | Area at `ortho.scale = 1.0` |
|---|---|---|
| 1280×720 | 16:9 | 1920×1080 |
| 1920×1080 | 16:9 | 1920×1080 |
| 2560×1080 | 21:9 | 2560×1080 |
| 1440×1080 | 4:3 | 1440×1080 |

## Window

The primary window starts in `WindowMode::BorderlessFullscreen(MonitorSelection::Primary)`
with `PresentMode::AutoVsync`. No windowed/fullscreen toggle exists.

## Zoom

- Zoom is the orthographic `scale`: higher `scale` shows more world.
- The Game clamps `scale` to `min_zoom`/`max_zoom` from `game.ron`.
- The Ship Editor clamps `scale` to `[0.1, 5.0]`.
- Visible height at any zoom is `VIEW_HEIGHT * ortho.scale` (for example `scale = 0.5`
  shows 540 world units tall, `scale = 2.0` shows 2160).

## Pan

- Game middle-drag and Ship Editor left-drag translate the camera by
  `mouse_delta * ortho.scale` (the editor multiplies by its `0.5` feel factor).
- `ortho.scale` equals world-units-per-pixel only when the logical viewport height equals
  `VIEW_HEIGHT` (1920×1080 at 100% DPI). On that design frame, drag tracking is 1:1 with
  the cursor at every zoom level.
- Other logical heights or DPI scales drift: above 1080 the camera overshoots the cursor,
  below 1080 it lags. This is accepted for the fullscreen design target, not a defect.

## Off-screen indicators

Edge arrows for off-screen ships stay world-sized (`45.0 * scale.sqrt()`) and are clamped
to window pixels. They do not adapt to screen size.

## Related code

| Path | Responsibility |
|---|---|
| `constant.rs` | `VIEW_HEIGHT` |
| `system/mod.rs` | camera spawn + projection |
| `main.rs` | borderless fullscreen window |
| `control/system.rs` | Game zoom, pan, follow |
| `scene/editor/input.rs` | Editor pan and zoom |
| `content/ship/indicator/mod.rs` | off-screen arrows |
