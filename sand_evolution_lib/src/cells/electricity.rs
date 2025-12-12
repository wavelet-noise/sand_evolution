use crate::cs::{self, PointType};

use super::{void::Void, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Electricity;
impl Electricity {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        60
    }
}
impl CellTrait for Electricity {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
        if prng.next() > 110 {
            let top = cs::xy_to_index(i, j + 1);
            let bot = cs::xy_to_index(i, j - 1);
            let left = cs::xy_to_index(i + 1, j);
            let right = cs::xy_to_index(i - 1, j);

            let arr = [top, left, right, bot];
            let cc = arr[(prng.next() % 4) as usize];
            let rand_v = container[cc];

            if rand_v == Void::id() {
                container[cc] = Electricity::id();
            }
        } else {
            container[cur] = Void::id();
        }
    }

    fn den(&self) -> i8 {
        0
    }
    fn casts_shadow(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "electricity"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
