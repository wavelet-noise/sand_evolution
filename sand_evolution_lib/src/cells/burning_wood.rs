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
        // Горящее дерево выделяет тепло
        if let Some(temp_ctx) = temp_context {
            (temp_ctx.add_temp)(i, j + 1, 2.0); // верх
            (temp_ctx.add_temp)(i, j - 1, 2.0); // низ
            (temp_ctx.add_temp)(i + 1, j, 2.0); // право
            (temp_ctx.add_temp)(i - 1, j, 2.0); // лево
        }
        if prng.next() > 200 {
            return;
        }

        if prng.next() > 250 {
            container[cur] = BurningCoal::id();
            //prng.add_carb();
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let topl = cs::xy_to_index(i - 1, j + 1);
        let topr = cs::xy_to_index(i + 1, j + 1);

        let arr = [top, topl, topr];
        let cc = arr[(prng.next() % 3) as usize];
        let top_v = container[cc];

        if top_v == Void::id() {
            container[cc] = fire::id();
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
