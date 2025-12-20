use crate::cs::PointType;

use super::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Stone;
impl Stone {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        255
    }
}
impl CellTrait for Stone {
    fn update(
        &self,
        _: PointType,
        _: PointType,
        _: usize,
        _: &mut [CellType],
        _: &CellRegistry,
        _: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
    }

    fn stat(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "stone"
    }
    fn id(&self) -> CellType {
        255
    }
    fn thermal_conductivity(&self) -> f32 {
        0.4
    }
    fn display_color(&self) -> [u8; 3] {
        [200, 200, 200]
    }
}
