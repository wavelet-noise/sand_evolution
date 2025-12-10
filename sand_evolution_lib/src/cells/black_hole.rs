use crate::cs::{self, PointType};

use super::{void::Void, CellRegistry, CellTrait, CellType, Prng};

pub struct BlackHole;

impl BlackHole {
    pub const fn new() -> Self {
        Self
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn id() -> CellType {
        80
    }
}

impl CellTrait for BlackHole {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        _cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        _prng: &mut Prng,
    ) {
        let neighbors = [
            cs::xy_to_index(i, j + 1),
            cs::xy_to_index(i, j - 1),
            cs::xy_to_index(i + 1, j),
            cs::xy_to_index(i - 1, j)
        ];

        for &idx in &neighbors {
            if container[idx] != Self::id() {
                container[idx] = Void::id();
            }
        }
    }

    fn den(&self) -> i8 {
        127
    }

    fn stat(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "black_hole"
    }

    fn id(&self) -> CellType {
        Self::id()
    }
}

