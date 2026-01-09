use crate::cells::salty_water::SaltyWater;
use crate::cs::PointType;

use super::{
    helper::sand_falling_helper, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
};

pub struct Salt;
impl Salt {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        13
    }
}
impl CellTrait for Salt {
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
        10
    }
    fn dissolve(&self) -> CellType {
        SaltyWater::id()
    }
    fn thermal_conductivity(&self) -> f32 {
        0.7
    }
    fn id(&self) -> CellType {
        13
    }
    fn name(&self) -> &str {
        "salt"
    }
    fn display_color(&self) -> [u8; 3] {
        [204, 204, 204]
    }
}

