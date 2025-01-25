# sand_evolution

[![deploy](https://github.com/wavelet-noise/sand_evolution/actions/workflows/push_to_master.yml/badge.svg?branch=master)](https://github.com/wavelet-noise/sand_evolution/actions/workflows/push_to_master.yml)

Play: https://wavelet-noise.github.io/sand_evolution/

You can initialize the simulation with a custom starting image. Use the save query parameter in the URL to pass in a PNG image that will be used as the initial state for your simulation.

Example png save:
https://wavelet-noise.github.io/sand_evolution/?save=https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/main/exported_example.png

And also with level script in txt file:
https://wavelet-noise.github.io/sand_evolution/?save=https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/main/empty_box.png&script_file=https://raw.githubusercontent.com/wavelet-noise/sand_evolution_maps/refs/heads/main/zeus2.rhai

Compilation for browsers: 
```wasm-pack build --release --target web sand_evolution_lib```

Regular compilation:
```cargo run --release```
