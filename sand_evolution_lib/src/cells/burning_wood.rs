use super::{*, TemperatureContext};
use crate::cs::{self, PointType};

pub const fn new() -> Cell {
    Cell
}
pub fn boxed() -> Box<Cell> {
    Box::new(new())
}

pub fn id() -> CellType {
    6
}

pub struct Cell;
impl CellTrait for Cell {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        // Burning wood releases heat
        if let Some(temp_ctx) = temp_context {
            (temp_ctx.add_temp)(i, j + 1, 1.0); // top
            (temp_ctx.add_temp)(i, j - 1, 1.0); // bottom
            (temp_ctx.add_temp)(i + 1, j, 1.0); // right
            (temp_ctx.add_temp)(i - 1, j, 1.0); // left
        }
        // Spawn "little fire" sparks independently from burn progression.
        // (My previous change accidentally made sparks rarer by gating this under progress.)
        if prng.next() <= 150 {
            let top = cs::xy_to_index(i, j + 1);
            let topl = cs::xy_to_index(i - 1, j + 1);
            let topr = cs::xy_to_index(i + 1, j + 1);

            let arr = [top, topl, topr];
            let cc = arr[(prng.next() % 3) as usize];
            let top_v = container[cc];

            if top_v == Void::id() {
                // Don't spawn fire every time: avoid runaway spread.
                if prng.next() > 210 {
                    container[cc] = fire::id();
                }
            }
        }
        if prng.next() > 25 {
            return;
        }

        // Char to coal only rarely per progress step.
        // With threshold 253 -> 2/256 ~= 0.78% per progress step.
        // Combined with the progress gate above gives ~22-23s expected lifetime.
        if prng.next() > 253 {
            container[cur] = Coal::id();
            return;
        }
    }

    fn stat(&self) -> bool {
        true
    }

    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> &str {
        "burning wood"
    }
    fn id(&self) -> CellType {
        6
    }
}
