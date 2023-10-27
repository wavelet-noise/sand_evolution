use crate::cells::crushed_ice::CrushedIce;
use crate::cs::{self, PointType};

use super::{void::Void, water::Water, CellRegistry, CellTrait, CellType, Prng};

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
        _pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        if prng.next() > 1 {
            return;
        }

        if prng.next() > 20 {
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let bot = cs::xy_to_index(i, j - 1);
        let left = cs::xy_to_index(i + 1, j);
        let right = cs::xy_to_index(i - 1, j);

        let arr = [top, left, right, bot];
        let cc = arr[(prng.next() % 4) as usize];
        let top_v = container[cc];

        if top_v == Void::id() || top_v == Water::id() {
            if prng.next() > 1 {
                container[cur] = Water::id();
            } else {
                container[cur] = CrushedIce::id();
            }
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
    fn name(&self) -> &str {
        "ice"
    }
}
