# zest-gui

`zest-gui` is the top-level convenience crate for the `zest` GUI framework family.

It re-exports:

- `zest-core`
- `zest-theme`
- `zest-widget`
- `zest-simulator` when the `simulator` feature is enabled

It also provides:

- a `prelude` module that pulls together the commonly used framework types
- a `time` re-export from `zest-core`
- an optional desktop `net` module
- `run::<A>(title)` as the default simulator entry point when the `simulator` feature is enabled

## Features

### `simulator`

Enables the default desktop runner by pulling in `zest-simulator`.

When this feature is enabled, applications can use:

```rust
zest_gui::run::<App>("My App").await;
```

### `net`

Enables the desktop async HTTP shim built on:

- `embassy-time`
- `reqwest`
- `serde`

The `net` module exists for desktop development; the source comments describe it as a compatibility layer that hides a blocking worker-thread implementation behind an async API.

## Prelude

The prelude re-exports framework types from the lower-level crates, including:

- application/runtime types from `zest-core`
- theme types from `zest-theme`
- concrete widgets from `zest-widget`
- common `embedded-graphics` geometry and color types

## Basic usage

```rust
use zest_gui::prelude::*;

struct App;
```

With the `simulator` feature enabled, the crate provides a ready-made desktop entry point:

```rust
#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    zest_gui::run::<App>("My App").await;
}
```

For custom hardware, construct your own `Platform` and call `Runtime::<A>::new().run(platform).await` directly.

## Package name and crate name

The package published to crates.io is `zest-gui`.

In Rust code, that package is imported as `zest_gui` unless the dependency is aliased in `Cargo.toml`.
