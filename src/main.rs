use std::ptr::null;

fn main() {
    pollster::block_on(sand_evolution_lib::run(2000.0, 1000.0, null(), 0));
}
