fn main() {
    pollster::block_on(sand_evolution_lib::run(
        2000.0,
        1000.0,
        Default::default(),
        "".to_owned()
    ));
}
