# zest-core

`zest-core` is the foundation crate for the `zest` GUI framework family.

It defines the application contract, the async runtime, the platform abstraction, the widget trait, event types, layout primitives, scrolling types, focus state, and dirty-region rendering support.

The crate is `no_std` outside tests and uses `alloc`.

## Main pieces

### Application model

The application surface is built around:

- `Application`
- `Task`
- `Subscription`
- `Recipe`
- `ScreenView`
- `Runtime`

`Application::init()` constructs the initial application state and startup task. `Application::update(...)` processes messages. `Application::view()` returns the active screen.

## Platform model

Backends implement `Platform`, which provides:

- `next_event()`
- `render_with(...)`
- `render_with_dirty(...)`
- `viewport()`
- `capabilities()`

Rendering uses a `Renderer` abstraction, and backends can expose `PlatformCapabilities` to tell the runtime whether clipping and partial flush are supported.

## Widget model

`zest-core` defines the object-safe `Widget<C, M>` trait and the boxed `Element` wrapper used to build heterogeneous trees.

The widget contract includes:

- measurement and arrangement
- touch handling
- semantic action routing
- focus participation
- drawing

## Input and actions

The core event surface includes:

- `InputEvent`
- `TouchEvent` / `TouchPhase`
- `KeyEvent` / `Key`
- `EncoderEvent`
- `UiAction`

The runtime also exposes focus-related types:

- `WidgetId`
- `FocusState`
- `FocusDirection`

## Layout, scrolling, and dirty regions

The crate exports shared layout and interaction primitives:

- `Constraints`
- `Length`
- `Horizontal`
- `Vertical`
- `UNBOUNDED`
- `ScrollState`
- `ScrollMsg`
- `ScrollDirection`
- `ScrollbarMode`
- `SnapMode`
- `GesturePhase`
- `tick_task(...)`
- `DirtyRegion`
- `PlatformCapabilities`

## Running an application

Applications are driven by `Runtime::<A>::new().run(platform).await`.

```rust
use zest_core::Runtime;

// Runtime::<App>::new().run(platform).await;
let _runtime = Runtime::<YourApp>::new();
```

`Runtime::run(...)` rebuilds the current screen, arranges the widget tree, synchronizes focus and pressed state, waits on platform input / pending tasks / subscriptions, and redraws as needed.

## Renderer helpers

The crate exports:

- `Renderer`
- `DrawTargetRenderer`
- `RenderError`
- `arc_sin_cos(...)`

These are the pieces both widgets and backends use to draw into a target.

## When to use `zest-core`

Use this crate directly when you are:

- writing a custom `Platform`
- building widgets outside `zest-widget`
- integrating the runtime with your own hardware backend
- working with the framework without the top-level convenience crate
