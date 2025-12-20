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
        10
    }

    fn name(&self) -> &str {
        "earth"
    }

    fn id(&self) -> CellType {
        Self::id()
    }
    /// Грунт — сыпучий, немного лучше проводит тепло, чем песок.
    fn thermal_conductivity(&self) -> f32 {
        0.6
    }
    fn display_color(&self) -> [u8; 3] {
        [120, 72, 35]
    }
}


