# zest

zest is a Rust GUI framework for small embedded displays. It is built around a message-driven application model, a transient widget tree, and a desktop simulator that is useful enough to do real day-to-day work before you move to hardware.

The overall shape is closer to iced or libcosmic than to a callback-heavy toolkit. Applications return messages, screens build views, and the runtime owns the event loop.

## Workspace layout

- **`zest-core`** — the application contract, runtime, input types, focus state, scroll state, dirty-region tracking, and renderer/platform traits
- **`zest-theme`** — palettes, typography, component styling, and preset themes
- **`zest-widget`** — the widget library and example programs
- **`zest-simulator`** — the desktop backend built on `embedded-graphics-simulator`, SDL2, and `tiny-skia`
- **`zest`** — the top-level crate that re-exports the rest and provides the default simulator entry point

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
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }
}
```

`init` builds the initial state and optional startup work. After that, all state changes flow through `update`.

### Screens and widgets

- `Application::view()` returns the active screen.
- `ScreenView::view(&self) -> Element<'_, C, M>` builds a fresh widget tree for the current frame.
- Widgets are transient. Cross-frame state lives in the application, the screen, or host-owned state objects such as `ScrollState`.
- The runtime re-applies interaction state like pressed visuals and focus each time it rebuilds the tree.

That transient model is the main architectural fact to keep in mind when reading the code. Focus, pressed state, scroll momentum, and similar interaction state do not live inside persistent widget instances because widget instances are not persistent.

### Platform

```rust
trait Platform {
    type Color: PixelColor;
    type Error;

    async fn next_event(&mut self) -> Option<InputEvent>;
    async fn render_with<F>(&mut self, draw: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut dyn Renderer<Self::Color>) -> Result<(), RenderError>;

    async fn render_with_dirty<F>(&mut self, dirty: &DirtyRegion, draw: F) -> Result<(), Self::Error>
    where
        F: FnOnce(&mut dyn Renderer<Self::Color>) -> Result<(), RenderError>;

    fn viewport(&self) -> Size;
    fn capabilities(&self) -> PlatformCapabilities;
}
```

Backends are user-defined. The simulator is the reference desktop backend; hardware support comes from implementing `Platform` around a display driver and whatever input source you have available.

The platform surface now includes a small capability layer and a dirty-region render hook. Full-frame redraw is still the universal fallback, but backends can opt into clipped redraw and partial flush behavior without changing widget code.

### Runtime

The runtime rebuilds the active screen, arranges it, re-synchronizes focus and pressed state, and then waits on the next platform event, pending task, or subscription event.

Input, async work, and subscriptions all feed back through the same `update` path via `embassy_futures::select3`.

The runtime also tracks what parts of the frame are dirty. If nothing visible changed, it skips the next draw. If only focus moved and the backend supports clipping, it can redraw only the dirty rectangles instead of repainting the full surface every time.

## Input, focus, and actions

`InputEvent` now includes:

- `Touch`
- `Key`
- `Encoder`
- `Action`

The runtime owns focus through stable `WidgetId` values. Today that means:

- `Tab` advances focus
- mouse wheel / encoder movement advances or reverses focus
- `Enter` activates the focused widget
- `Escape` routes cancel
- arrow keys route directional semantic actions
- touch-down assigns focus when it lands on a focusable widget

This is still an early system, but it is no longer touch-only and it is no longer just a proof of concept.

Focus/action support is currently wired into:

- `Button`
- `Checkbox`
- `RadioButton`
- `Switch`
- `Slider`
- `SpinButton`
- `Menu`
- `List`
- `Dropdown`
- `TabBar`
- `TextArea`
- `Roller`
- `Table`
- forwarding containers and wrappers such as `Column`, `Row`, `Container`, `Stack`, and `Scrollable`

There is still follow-on work to do on some higher-level or more specialized widgets, but the main interactive controls now participate in the same focus and semantic-action path.

## Rendering and dirty-region tracking

zest still uses the simple transient-tree model: widgets are rebuilt and drawn from scratch for each frame that needs rendering.

What changed is that the runtime no longer has to assume every pass is a full-screen redraw.

- `zest-core` now exposes `DirtyRegion` and `PlatformCapabilities`
- the runtime tracks whether the next frame needs no redraw, a full redraw, or only dirty rectangles
- clipped redraw is used when the backend says it can support it
- the simulator supports partial flushes and can optionally outline dirty rectangles

The dirty-region model is intentionally small right now. It is enough to support partial redraw plumbing without turning every widget into a region calculator.

## Current status

| Area | State | Notes |
|---|---|---|
| Application model | Solid | `init`, `update`, `view`, `Task`, and `Subscription` are all in place |
| View model | Transient | `ScreenView::view(&self)` builds a fresh `Element` tree every frame |
| Input | Mixed-input | touch, keyboard, encoder, and semantic actions exist in core |
| Focus/actions | Broad first pass | stable `WidgetId`-based focus and directional action routing |
| Text editing | Usable, still early | `TextArea` is focusable and action-aware, but the text foundation still needs shared editing primitives |
| Rendering | Dirty-region aware | full redraw still works everywhere; clipped redraw and partial flush are now part of the core model |
| Simulator | Strong local tool | keyboard input, encoder-like wheel input, partial redraw, and optional dirty-region visualization |
| Widget catalog | Broad | 43 runnable examples and a large widget surface |
| Theme system | Broad preset coverage | 19 preset theme modules are exported, plus a custom builder for project-specific palettes |

## Theme presets

`zest-theme::theme` currently exports these preset modules:

- `light`, `dark`
- `dracula`, `dracula_at_night`
- `nord`
- `catppuccin_latte`, `catppuccin_frappe`, `catppuccin_macchiato`, `catppuccin_mocha`
- `tokyo_night`, `tokyo_night_storm`, `tokyo_night_light`
- `kanagawa_wave`, `kanagawa_dragon`, `kanagawa_lotus`
- `moonfly`, `nightfly`
- `oxocarbon`
- `ferra`

Each preset module exposes a `THEME` constant as `Theme<'static, Rgb888>`. Use `convert_theme(...)` to convert it to the color type your target uses.

For custom palettes, use `theme::custom::CustomBuilder`.

## Examples

`zest-widget/examples/` currently contains **43 runnable examples**. A few good starting points:

| Name | What it covers |
|---|---|
| `counter_app` | full application flow with tasks and subscriptions |
| `app` | a larger multi-screen demo app |
| `focus_navigation` | focus traversal and semantic activation across several widget families |
| `checkbox`, `radio`, `switch`, `slider`, `spin_button` | the basic control set with focus ids wired in |
| `dropdown`, `menu`, `list`, `roller`, `table` | selection widgets and structured data/navigation widgets |
| `text_area` | host-driven text editing with `TextArea` and `Keyboard` |

All of the examples use the same general shape:

```rust
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest::run::<App>("zest - example").await;
}
```

### Running examples

```sh
cargo run -p zest-widget --example focus_navigation
cargo run -p zest-widget --example text_area
cargo run -p zest-widget --example table
cargo run -p zest-widget --example spin_button
```

The simulator backend depends on SDL2 being available on the machine that runs it.

### Simulator dirty-region overlay

If you want to inspect the new dirty-region path, construct the simulator directly instead of using `zest::run`:

```rust
let platform = zest::zest_simulator::SimulatorPlatform::builder("demo")
    .show_dirty(true)
    .build();

zest::zest_core::Runtime::<App>::new().run(platform).await;
```

That keeps the normal simulator behavior but outlines dirty rectangles after partial redraws.

## Local validation

These are the local checks that currently reflect the real state of the repository:

```sh
cargo check --workspace
cargo check -p zest-widget --examples
cargo check -p zest --features simulator
cargo fmt --all --check
```

Strict `cargo clippy --workspace --all-targets -D warnings` is still **not** clean at the moment because `zest-theme` has pre-existing lint and doc issues. That needs to be fixed, but the documented workflow should stay honest about the current baseline.

## What is still missing

The main remaining gaps are no longer the same ones this work started with.

What still stands out now:

- richer shared text/editing primitives instead of keeping cursor/edit semantics mostly inside examples and host code
- broader follow-through on the remaining specialized widgets
- more hardware-facing validation and board-specific examples
- continued cleanup of theme coverage and repository-wide lint debt

## Caveats

The project is still moving. Public APIs, some widget behaviors, and theme coverage are not settled enough to promise long-term stability yet.
