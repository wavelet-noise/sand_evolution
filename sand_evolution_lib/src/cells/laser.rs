use crate::cells::helper::fluid_falling_helper;
use crate::cs::{self, PointType};

use super::{void::Void, CellRegistry, CellTrait, CellType, Prng};

pub struct Laser;
impl Laser {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        62
    }
}
impl CellTrait for Laser {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        if prng.next() < 200 {
            return
        } else {
            container[cur] = Void::id();
        }
    }

    fn den(&self) -> i8 {
        0
    }
    fn name(&self) -> &str {
        "laser"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
