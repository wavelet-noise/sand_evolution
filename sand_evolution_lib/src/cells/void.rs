use crate::cs::PointType;

use super::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Void;
impl Void {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        0
    }
}
impl CellTrait for Void {
    fn update(
        &self,
        _: PointType,
        _: PointType,
        _: usize,
        _: &mut [u8],
        _: &CellRegistry,
        _: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
    }
    fn name(&self) -> &str {
        "void"
    }
}
