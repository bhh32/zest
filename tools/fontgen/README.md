# fontgen

Generates zest's default bitmap mono font family (`zest-theme/src/font.rs`
and `zest-theme/fonts/zest_mono*.raw`) by rasterizing a monospace TTF into
embedded-graphics' 1-bpp `MonoFont` atlas format.

```sh
cd tools/fontgen
cargo run                      # uses DejaVu Sans Mono (system)
cargo run -- /path/to/Mono.ttf # or a custom monospace TTF
```

Edit the `subset()` function to change which Unicode code points are
included (only code points present in the source font are emitted).
Outputs are written relative to this crate, so run it from anywhere.
