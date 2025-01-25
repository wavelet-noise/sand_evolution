use crate::cells::helper::fluid_falling_helper;
use crate::cs::{self, PointType};

use super::{void::Void, CellRegistry, CellTrait, CellType, Prng};

pub struct Plasma;
impl Plasma {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        61
    }
}
impl CellTrait for Plasma {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        if prng.next() > 200 && fluid_falling_helper(self.den(), i, j, container, _pal_container, cur, prng, 10)
        {
            return;
        }
        if prng.next() > 110 {
            let top = cs::xy_to_index(i, j + 1);
            let bot = cs::xy_to_index(i, j - 1);
            let left = cs::xy_to_index(i + 1, j);
            let right = cs::xy_to_index(i - 1, j);

            let arr = [top, left, right, bot];
            let cc = arr[(prng.next() % 4) as usize];
            let rand_v = container[cc];

            if rand_v == Void::id() {
                container[cc] = Plasma::id();
            }
        } else {
            container[cur] = Void::id();
        }
    }

    fn den(&self) -> i8 {
        1
    }
    fn name(&self) -> &str {
        "plasma"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
