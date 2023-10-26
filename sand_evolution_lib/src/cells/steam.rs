use super::{helper::fluid_flying_helper, *};
use crate::cs::PointType;

pub struct Steam;
impl Steam {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        3
    }
}

impl CellTrait for Steam {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, prng);
    }

    fn den(&self) -> i8 {
        -1
    }

    fn name(&self) -> &str {
        "steam"
    }

    fn id(&self) -> CellType {
        3
    }
}
