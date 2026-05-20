# zest

A retained-mode GUI framework for embedded touchscreen MCUs. Targets the CYD R3 (ESP32-WROOM-32), Pico W, and ESP32-S3 WROOM with RGB565 displays. `no_std + alloc`. Edition 2024, MSRV 1.85.

Built on embassy as the async runtime, with an architectural shape modelled on iced and libcosmic.

## Workspace layout

- **`zest-core`** — application contract: `Application`, `ScreenView`, `Platform`, `Runtime` (async), `Recipe`, `Subscription`, `Task`, and a `time` module with the built-in `every` helper.
- **`zest-theme`** — `Theme<'a, C>` with grouped `Container` / `Component` / `Palette` / `Spacing` / `CornerRadii` types. 20 preset themes (`theme::dark`, `theme::dracula`, etc.) defined in `Rgb888` with `convert_theme<C>` for any target color.
- **`zest-widget`** — Widget library: `Widget` trait, `Element`, `Button`, `Column`, `Row`, `Grid`, `Container`, `Keyboard`. Trait declaration and `pub mod` declarations live together in `widget.rs` (no `mod.rs` anywhere).
- **`zest-simulator`** — Desktop `Platform` implementation backed by `embedded-graphics-simulator` + SDL2 + `tiny-skia`.
- **`zest`** — Top-level crate re-exporting the others. With the `simulator` feature, provides `zest::run::<App>(title)` for the default desktop case.

## Architecture

### Application — libcosmic-shaped

```rust
trait Application: Sized + 'static {
    type Message: Clone + 'static;
    type Color: PixelColor;
    type Screen: ScreenView<Self::Color, Self::Message>;

    fn init() -> (Self, Task<Self::Message>);
    fn update(&mut self, msg: Self::Message) -> Task<Self::Message>;
    fn view(&self) -> &Self::Screen;
    fn view_mut(&mut self) -> &mut Self::Screen;
    fn subscription(&self) -> Subscription<Self::Message> { Subscription::none() }
}
```

The runtime calls `A::init()` once at startup to get the initial app value and a startup task. `update` is the only place global state mutates in response to messages.

### Platform — async, user-pluggable

```rust
trait Platform {
    type Color: PixelColor;
    type Display: DrawTarget<Color = Self::Color>;
    type Error;
    async fn next_event(&mut self) -> Option<InputEvent>;
    async fn render_with<F>(&mut self, draw: F) -> Result<(), Self::Error>
    where F: FnOnce(&mut Self::Display) -> Result<(), <Self::Display as DrawTarget>::Error>;
    fn viewport(&self) -> Size;
}
```

Backend is user-defined. zest ships `zest-simulator` for desktop development; for hardware, users implement `Platform` themselves wrapping their display driver (mipidsi, etc.) and touch driver. `render_with`'s closure form lets the platform handle compositing (paged framebuffers, half-height blits, double buffering) while exposing a single full-resolution `DrawTarget` to widgets.

### Runtime — async, embassy-driven

```rust
zest::run::<MyApp>("My App").await;
```

The runtime owns the event loop. Each iteration uses `embassy_futures::select3` to concurrently await:

1. The next platform input event
2. Any pending `Task` future (multiple via `Task::batch`)
3. The active `Subscription`'s next future

Whichever fires first feeds back through `Application::update`. After every processed message, `Application::subscription()` is re-called and compared by identity (see Recipe below) — matching identity keeps the existing future running unbroken, differing identity replaces.

Executor-agnostic at the type level — examples use `#[embassy_executor::main]` with `arch-std` for desktop, but any executor that polls a `Future<Output = ()>` works.

### Task

Side-effecting async work, returned from `init` or `update`. Three constructors:

```rust
Task::none()                       // no work
Task::perform(async { ... })       // future producing Option<Msg>
Task::future(async { ... })        // future always producing Msg
Task::batch([t1, t2, t3])          // concurrent — each task's message flows independently
```

### Subscription + Recipe

Long-running message sources. The runtime refreshes after every message; identity comes from the underlying `Recipe`:

```rust
trait Recipe: Hash + 'static {
    type Message: 'static;
    fn next(&mut self) -> Option<BoxFuture<Self::Message>>;
}
```

Identity is `TypeId::of::<R>()` + `Hash` of the recipe's fields. Refactor-stable; same recipe value across refreshes keeps the existing future running.

Most users never write a Recipe — `zest::time::every` wraps the built-in `Tick<M>` recipe:

```rust
fn subscription(&self) -> Subscription<Msg> {
    if self.auto_tick {
        zest::time::every(Duration::from_secs(1), Msg::Tick)
    } else {
        Subscription::none()
    }
}
```

Power users implementing custom event sources (websockets, GPIO interrupts, sensor streams) write their own `Recipe`.

### Application + Screens

- Widgets are **persistent objects** owned by screens (current model — see "Future directions" below).
- Each screen is a struct holding its widget tree as fields, implementing `ScreenView`.
- `Application::view_mut` exists for runtime-internal event dispatch and layout; not intended for user code.

### Layout

Two-pass: `Widget::measure(&mut self, Constraints) -> Size` then `Widget::arrange(&mut self, Rectangle)`. The runtime runs these on first frame and on viewport changes.

### Transient state

Pressed/hovered state is cleared via `Widget::sweep(&mut self)`. The runtime calls sweep on `TouchPhase::Up`, so pressed visuals are visible from Down to Up.

## Theme presets

Fully implemented: `light`, `dark` (default), `dracula`, `dracula_at_night`, `nord`, `tokyo_night`.

Stubbed (re-export `dark` until palettes are transcribed from canonical sources): `catppuccin_latte` / `frappe` / `macchiato` / `mocha`, `tokyo_night_storm`, `tokyo_night_light`, `kanagawa_wave` / `dragon` / `lotus`, `moonfly`, `nightfly`, `oxocarbon`, `ferra`. Plus `theme::custom::CustomBuilder` for user-defined themes.

## Examples

Seven runnable examples in `zest-widget/examples/`:

| Name | Demonstrates |
|---|---|
| `counter_app` | Full `Application` pattern: `init`, `update`, `view`, `subscription`. Also `Task::perform` (delayed Reset), `Task::batch` (three concurrent timers on "Boom!"), `zest::time::every` (toggleable auto-tick subscription with refresh) |
| `button` | Button states: normal, success, destructive, disabled, accent |
| `column` | Equal-height vertical stack |
| `row` | Weighted horizontal layout (1× / 2× / 3×) |
| `grid` | 3×4 numeric keypad |
| `container` | Nested containers with progressive padding |
| `keyboard` | On-screen QWERTY keyboard bridging `KeyboardEvent` → app `Msg` |

All examples follow the same shape:

```rust
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest — example_name").await;
}
```

### Running the examples

The simulator needs SDL2 (`sudo dnf install SDL2-devel` on Fedora, `sudo apt install libsdl2-dev` on Debian/Ubuntu).

```sh
cargo run -p zest-widget --example counter_app
cargo run -p zest-widget --example button
cargo run -p zest-widget --example keyboard
# ...etc
```

## Future directions

The widget model is currently **persistent**: widgets are constructed once on the screen struct and reused across frames. This is in tension with the rest of the framework, which mirrors iced/libcosmic (immutable `view`, all mutation through `update`). The natural next step is to migrate widgets to the **transient** model: `ScreenView::view(&self) -> Element<'_, C, M>` returns a fresh widget tree built from screen data each frame; widgets borrow from screen state via `Cow<'a, str>`; `view_mut` is removed.

This will also enable a `TextBox` widget that reads its content from screen-owned `String` state via `Cow<'a, str>`, with the `Keyboard` widget's `on_input(msg_fn)` emitting user messages that flow through `update` to mutate that state.

Separately, a `zest-desktop` crate is planned to replace `zest-simulator` for desktop deployment — winit + tiny-skia instead of SDL2, with the same `Platform` trait surface. Application code unchanged.

## Known caveats
This framework is under HEAVY changes and the author does NOT guarantee or offer backward compatibility between releases.
DO NOT USE THE MAIN BRANCH FOR PRODUCTION USE!!!

