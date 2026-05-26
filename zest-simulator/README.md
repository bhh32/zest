# zest-simulator

`zest-simulator` is the desktop `Platform` implementation for the `zest` GUI framework family.

It renders with `tiny-skia` into an RGBA pixmap, converts the output to `Rgb565`, and displays it through `embedded-graphics-simulator` and SDL2.

The simulator implements `zest_core::Platform<Color = Rgb565>`.

## What it provides

- `SimulatorPlatform`
- `SimulatorPlatformBuilder`
- default desktop settings for a 320×240 RGB565 display
- SDL-backed event polling
- touch-style input through mouse press / move / release
- keyboard input mapped into `KeyEvent`
- mouse-wheel input mapped into `EncoderEvent`
- dirty-region rendering support
- optional dirty-rectangle overlay output

## Defaults

The crate exports these defaults:

- `DEFAULT_WIDTH = 320`
- `DEFAULT_HEIGHT = 240`
- `DEFAULT_SCALE = 2`
- `DEFAULT_PIXEL_SPACING = 0`
- `DEFAULT_POLL_MS = 16`

## Basic usage

```rust
use zest_simulator::SimulatorPlatform;

let platform = SimulatorPlatform::new("demo");
```

For custom configuration, use the builder:

```rust
use embedded_graphics::prelude::Size;
use zest_simulator::SimulatorPlatform;

let platform = SimulatorPlatform::builder("demo")
    .size(Size::new(320, 240))
    .scale(2)
    .pixel_spacing(0)
    .poll_ms(16)
    .show_dirty(true)
    .build();
```

## Input mapping

The simulator maps SDL events into `zest-core` input types:

- left mouse button down / move / up → `TouchEvent`
- supported keyboard keys → `KeyEvent`
- mouse wheel → `EncoderEvent`

## Rendering behavior

The simulator reports clipping and partial-flush support through `PlatformCapabilities`.

It implements both:

- `render_with(...)`
- `render_with_dirty(...)`

When `show_dirty(true)` is enabled, the simulator outlines dirty rectangles after partial redraws.

## Notes

- The simulator is fixed to `Rgb565`.
- The crate uses `std`.
- It is intended for desktop development and debugging, not embedded deployment.
