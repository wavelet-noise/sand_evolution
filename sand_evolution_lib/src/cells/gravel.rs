use crate::cs::PointType;

use super::{
    helper::sand_falling_helper, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
};

/// Gravel — сыпучий материал, тяжелее песка/земли.
pub struct Gravel;

impl Gravel {
    pub const fn new() -> Self {
        Self
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn id() -> CellType {
        19
    }
}

impl CellTrait for Gravel {
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
        12
    }

    fn name(&self) -> &str {
        "gravel"
    }

    fn id(&self) -> CellType {
        Self::id()
    }
}
