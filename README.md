# zest

zest is a retained-mode GUI framework for small embedded displays. It is `no_std + alloc`, uses Rust 2024, and currently targets Rust 1.85 or newer.

The runtime is built on embassy, and the overall shape is closer to iced/libcosmic than to a traditional callback-driven widget toolkit. Input is touch-only for now; keyboard/encoder support and a proper focus system are still on the roadmap.

## Workspace layout

- **`zest-core`** — application contract: `Application`, `ScreenView`, `Platform`, `Runtime` (async), `Recipe`, `Subscription`, `Task`, and a `time` module with the built-in `every` helper.
- **`zest-theme`** — `Theme<'a, C>` with grouped `Container` / `Component` / `Palette` / `Spacing` / `CornerRadii` types. 20 preset themes (`theme::dark`, `theme::dracula`, etc.) defined in `Rgb888` with `convert_theme<C>` for any target color.
- **`zest-widget`** — Widget library: `Widget` trait, `Element`, `Button`, `Column`, `Row`, `Grid`, `Container`, `Keyboard`. Trait declaration and `pub mod` declarations live together in `widget.rs` (no `mod.rs` anywhere).
- **`zest-simulator`** — Desktop `Platform` implementation backed by `embedded-graphics-simulator` + SDL2 + `tiny-skia`.
- **`zest`** — Top-level crate re-exporting the others. With the `simulator` feature, provides `zest::run::<App>(title)` for the default desktop case.

## Architecture

### Application

```rust
trait Application: Sized + 'static {
    type Message: Clone + 'static;
    type Color: PixelColor;
    type Screen: ScreenView<Self::Color, Self::Message>;

    fn init() -> (Self, Task<Self::Message>);
    fn update(&mut self, msg: Self::Message) -> Task<Self::Message>;
    fn view(&self) -> &Self::Screen;
    fn subscription(&self) -> Subscription<Self::Message> { Subscription::none() }
}
```

The runtime calls `A::init()` once at startup to get the initial application state and a startup task. After that, application-wide state changes flow through `update`.

### Platform

```rust
trait Platform {
    type Color: PixelColor;
    type Error;
    async fn next_event(&mut self) -> Option<InputEvent>;
    async fn render_with<F>(&mut self, draw: F) -> Result<(), Self::Error>
    where F: FnOnce(&mut dyn Renderer<Self::Color>) -> Result<(), RenderError>;
    fn viewport(&self) -> Size;
}
```

Backends are user-defined. zest ships `zest-simulator` for desktop development; on hardware, you implement `Platform` around your display driver and input source. The `render_with` closure leaves room for paged framebuffers, partial buffers, and double buffering without changing widget code.

### Runtime

```rust
zest::run::<MyApp>("My App").await;
```

The runtime owns the event loop. Each iteration uses `embassy_futures::select3` to concurrently await:

1. The next platform input event
2. Any pending `Task` future (multiple via `Task::batch`)
3. The active `Subscription`'s next future

Whichever branch resolves first feeds back through `Application::update`. After every processed message, `Application::subscription()` is called again and compared by identity (see Recipe below): if the recipe is unchanged, the existing future keeps running; if not, it is replaced.

At the type level, the runtime is executor-agnostic. The examples use `#[embassy_executor::main]` with `arch-std` on desktop, but anything that can poll a `Future<Output = ()>` will work.

### Task

Side-effecting async work returned from `init` or `update`. There are three constructors:

```rust
Task::none()                       // no work
Task::perform(async { ... })       // future producing Option<Msg>
Task::future(async { ... })        // future always producing Msg
Task::batch([t1, t2, t3])          // concurrent — each task's message flows independently
```

### Subscription + Recipe

Long-running message sources. The runtime refreshes subscriptions after every message; identity comes from the underlying `Recipe`:

```rust
trait Recipe: Hash + 'static {
    type Message: 'static;
    fn next(&mut self) -> Option<BoxFuture<Self::Message>>;
}
```

Identity is `TypeId::of::<R>()` plus the `Hash` of the recipe fields. The same recipe value keeps the existing future alive across refreshes.

Most applications never need to implement `Recipe` directly. `zest::time::every` wraps the built-in `Tick<M>` recipe:

```rust
fn subscription(&self) -> Subscription<Msg> {
    if self.auto_tick {
        zest::time::every(Duration::from_secs(1), Msg::Tick)
    } else {
        Subscription::none()
    }
}
```

If you need a custom event source such as a websocket, GPIO interrupt stream, or sensor feed, you can write your own `Recipe`.

### Screens

- `Application::view()` returns the active screen.
- Each screen implements `ScreenView`.
- `ScreenView::view(&self) -> Element<'_, C, M>` builds a fresh widget tree for the current frame.
- Cross-frame state lives in the application, the screen, or host-owned state objects such as `ScrollState`, not inside widget instances.

### Layout

Widgets use a measure/arrange-style contract. On each loop iteration, the runtime rebuilds the tree, arranges it against the current viewport, draws it, and routes input through it.

### Input today

- `InputEvent` is currently touch-only.
- Pressed visuals are touch-driven via `mark_pressed` during the active gesture.
- There is not yet a focus tree, keyboard navigation path, or encoder/action system.

## Current status

| Area | Current state | Notes |
|---|---|---|
| Application model | Message-driven | `init`, `update`, `view`, `subscription`, `Task`, `Subscription` |
| View/widget model | Transient | `ScreenView::view(&self)` returns a fresh `Element` tree each frame |
| Input | Touch only | Keyboard/rotary support is planned |
| Focus/actions | Not there yet | No focus tree, traversal, or semantic action system today |
| Text editing | Early but usable | `TextArea` exists, but currently assumes mono-font / ASCII-friendly editing |
| Rendering | Full-frame oriented | `render_with` keeps backend compositing flexible; invalidation/partial redraw are still future work |
| Simulator | Mature enough for day-to-day use | SDL2 + `embedded-graphics-simulator` + `tiny-skia` |
| Widget catalog | Broad | More than 30 runnable examples cover controls, layout, scrolling, text, and demo apps |
| Theme system | Solid base | libcosmic-inspired tokens; some preset themes are still stubbed |

## Theme presets

Fully implemented: `light`, `dark` (default), `dracula`, `dracula_at_night`, `nord`, `tokyo_night`.

These presets still fall back to `dark` until their palettes are filled in from the canonical sources: `catppuccin_latte` / `frappe` / `macchiato` / `mocha`, `tokyo_night_storm`, `tokyo_night_light`, `kanagawa_wave` / `dragon` / `lotus`, `moonfly`, `nightfly`, `oxocarbon`, and `ferra`.

For custom palettes, use `theme::custom::CustomBuilder`.

## Examples

`zest-widget/examples/` contains more than 30 runnable examples. A few representative ones:

| Name | Demonstrates |
|---|---|
| `counter_app` | Full `Application` pattern: `init`, `update`, `view`, `subscription`. Also `Task::perform` (delayed Reset), `Task::batch` (three concurrent timers on "Boom!"), `zest::time::every` (toggleable auto-tick subscription with refresh) |
| `button` | Button states: normal, success, destructive, disabled, accent |
| `column` | Equal-height vertical stack |
| `row` | Weighted horizontal layout (1× / 2× / 3×) |
| `grid` | 3×4 numeric keypad |
| `container` | Nested containers with progressive padding |
| `keyboard` | On-screen QWERTY keyboard bridging `KeyAction` → app `Msg` |

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

- Extend `InputEvent` and the runtime beyond touch-only operation to support keyboard, rotary encoder, focus traversal, and semantic actions.
- Add a focus tree and consistent non-touch behavior across widgets.
- Improve text foundations beyond the current mono-font / ASCII-friendly assumptions used by `TextArea`.
- Add invalidation and partial redraw support so hardware backends can avoid full-frame work where possible.
- Continue evolving desktop support; a `zest-desktop` crate remains a possible future replacement for `zest-simulator`.

## Known caveats

This project is still changing quickly, and backward compatibility between releases is not guaranteed.

Do not treat `main` as production-stable.
