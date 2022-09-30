mod lib;
use lib::run;

fn main() {
    pollster::block_on(run());
}