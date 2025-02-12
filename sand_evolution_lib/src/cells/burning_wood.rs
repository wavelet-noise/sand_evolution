use super::*;
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
    ) {
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
