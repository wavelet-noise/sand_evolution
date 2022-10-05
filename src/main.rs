mod dimension;

fn main() {
    pollster::block_on(dimension::run());
}