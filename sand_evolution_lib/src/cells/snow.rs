use crate::cs::{PointType, self};

use super::{
    helper::{sand_faling_helper, fluid_falling_helper},
    water::{BaseWater, SaltyWater, Water},
    CellRegistry, CellTrait, CellType, Prng, void::Void,
};

pub struct Snow;
impl Snow {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        57
    }
}
impl CellTrait for Snow {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        if prng.next() > 128 && fluid_falling_helper(self.den(), i, j, container, pal_container, cur, prng, 10) {
            return;
        }

        if prng.next() > 1 {
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let bot = cs::xy_to_index(i, j - 1);
        let left = cs::xy_to_index(i + 1, j);
        let right = cs::xy_to_index(i - 1, j);

        let arr = [top, left, right, bot];
        let cc = arr[(prng.next() % 3) as usize];
        let top_v = container[cc];

        if top_v == Void::id() || top_v == Water::id() {
            container[cur] = Water::id();
        }
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn den(&self) -> i8 {
        5
    }
    fn id(&self) -> CellType {
        Self::id()
    }
    fn name(&self) -> String {
        "snow".to_owned()
    }
}