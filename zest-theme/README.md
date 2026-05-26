# zest-theme

`zest-theme` contains the theme types used by the `zest` crate family.

The crate is built around a grouped theme model:

- `Theme<'a, C>` as the top-level theme value
- `Container<C>` for region colors such as `background`, `primary`, and `secondary`
- `Component<C>` for interactive roles such as `button`, `accent`, `destructive`, and `success`
- `Palette<C>` for named primitive colors
- `Typography<'a>` for the display / heading / body / caption font roles
- `Spacing` and `CornerRadii` for shared layout values

`Theme` is generic over `embedded_graphics::pixelcolor::PixelColor`, and the preset theme modules are defined as `Theme<'static, Rgb888>` constants.

## What the crate exports

The public surface includes:

- `Theme`
- `Container`
- `Component`
- `Palette`
- `Typography`
- `Spacing`
- `CornerRadii`
- `ButtonAppearance`, `ButtonCatalog`, `ButtonClass`, and `Status`
- `convert_theme(...)`
- built-in mono font constants:
  - `FONT_ZEST_MONO`
  - `FONT_ZEST_MONO_DISPLAY`
  - `FONT_ZEST_MONO_HEADING`
  - `FONT_ZEST_MONO_CAPTION`

## Preset themes

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

Each preset module exports a `THEME` constant.

## Converting a preset to your display color

```rust
use embedded_graphics::pixelcolor::Rgb565;
use zest_theme::{Theme, convert_theme, theme::dark};

let theme: Theme<'static, Rgb565> = convert_theme(&dark::THEME);
```

## Building a custom theme

The crate also includes `theme::custom::CustomBuilder`, which starts from the dark or light preset and lets you replace individual fields.

```rust
use zest_theme::theme::custom::CustomBuilder;

let custom = CustomBuilder::from_dark().build();
```

## Notes

- The crate is `no_std` outside tests.
- Widgets do not consume the theme by reaching directly into every field; `Theme` implements catalog traits such as `ButtonCatalog`, and widget code uses those role-based lookups.
