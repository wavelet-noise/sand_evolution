use crate::cs::{PointType, self};

use super::{
    helper::sand_faling_helper,
    water::{BaseWater, SaltyWater, Water},
    CellRegistry, CellTrait, CellType, Prng, void::Void,
};

pub struct Ice;
impl Ice {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        55
    }
}
impl CellTrait for Ice {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
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

    fn heat_proof(&self) -> u8 {
        240
    }

    fn den(&self) -> i8 {
        10
    }
    fn id(&self) -> CellType {
        Self::id()
    }
    fn name(&self) -> String {
        "ice".to_owned()
    }
}
