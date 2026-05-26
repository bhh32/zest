# zest-widget

`zest-widget` is the standard widget library for the `zest` GUI framework family.

It builds on `zest-core` and `zest-theme` and exports the concrete widget set used by the examples and higher-level applications.

The crate is `no_std` outside tests.

## Widget set

The current public exports include:

### Layout and composition

- `Column`
- `Row`
- `Grid`
- `Container`
- `Stack`
- `Scrollable`
- `Space`
- `Divider`

### Text and text entry

- `Text`
- `TextArea`
- `Keyboard`

### Buttons and choice controls

- `Button`
- `ImageButton`
- `Checkbox`
- `RadioButton`
- `Switch`
- `Dropdown`
- `Menu`
- `TabBar`

### Value and status widgets

- `Slider`
- `SpinButton`
- `ProgressBar`
- `Spinner`
- `LED`
- `Scale`
- `Arc`

### Lists, tables, and scrolling widgets

- `List`
- `ListRow`
- `Table`
- `TableRow`
- `Roller`
- `Tileview`

### Graphics and media widgets

- `Image`
- `Canvas`
- `Chart`
- `Line`
- `Qr`
- `WeatherIcon`

### Higher-level surfaces

- `Calendar`
- `MessageBox`
- `Window`
- `Span`
- `SpanGroup`

## Shared types re-exported from `zest-core`

The crate also re-exports scrolling types for widget authors and applications:

- `ScrollState`
- `ScrollMsg`
- `ScrollDirection`
- `ScrollbarMode`
- `SnapMode`
- `tick_task(...)`

## Example programs

This crate contains a large example set under `zest-widget/examples/`.

The example registry currently includes examples such as:

- `counter_app`
- `app`
- `focus_navigation`
- `gauge`
- `text_area`
- `table`
- `dropdown`
- `roller`
- `calendar`
- `scroll_list`
- `scroll_row`
- `scroll_grid`
- `scroll_snap`

## Basic usage

The widgets are designed to be composed into an `Element<'_, C, M>` tree.

```rust
use zest_widget::{Button, Column, IntoElement};

let view = Column::new()
    .push(Button::new("OK"))
    .into_element();
```

## Notes

- `zest-widget` exports concrete widgets only; the `Widget` trait itself comes from `zest-core` and is re-exported here.
- Focus/action support is handled through the same widget tree that is built every frame.
- The crate is intended to pair with `zest-core::Runtime` directly or through the top-level `zest-gui` crate.
