use crate::cs::PointType;

use super::{
    helper::sand_falling_helper, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
};

/// Earth (soil) — рыхлый сыпучий материал, ведёт себя почти как песок.
pub struct Earth;

impl Earth {
    pub const fn new() -> Self {
        Self
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn id() -> CellType {
        18
    }
}

impl CellTrait for Earth {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }

    fn den(&self) -> i8 {
        // Чуть тяжелее песка, чтобы чаще оседала ниже.
        11
    }

    fn name(&self) -> &str {
        "earth"
    }

    fn id(&self) -> CellType {
        Self::id()
    }
}

