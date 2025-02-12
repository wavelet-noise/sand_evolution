use crate::cs::PointType;

use super::{burning_wood, gas::Gas, CellRegistry, CellTrait, CellType, Prng};

pub struct Wood;
impl Wood {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        50
    }
}
impl CellTrait for Wood {
    fn update(
        &self,
        _: PointType,
        _: PointType,
        _: usize,
        _: &mut [CellType],
        _: &CellRegistry,
        _: &mut Prng,
    ) {
    }

    fn stat(&self) -> bool {
        true
    }
    fn burnable(&self) -> u8 {
        burning_wood::id()
    }
    fn proton_transfer(&self) -> CellType {
        Gas::id()
    }
    fn name(&self) -> &str {
        "wood"
    }
    fn id(&self) -> CellType {
        50
    }
}
