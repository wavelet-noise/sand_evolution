# sand_evolution

[![deploy](https://github.com/wavelet-noise/sand_evolution/actions/workflows/push_to_master.yml/badge.svg?branch=master)](https://github.com/wavelet-noise/sand_evolution/actions/workflows/push_to_master.yml)

Play: https://wavelet-noise.github.io/sand_evolution/

You can pass save in png like this:
https://wavelet-noise.github.io/sand_evolution/?save=%22https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/main/exported_example.png%22

Compilation for browsers: 
```wasm-pack build --release --target web sand_evolution_lib```

Regular compilation:
```cargo run --release```
