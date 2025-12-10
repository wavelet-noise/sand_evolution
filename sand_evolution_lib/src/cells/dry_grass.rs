use crate::cs::PointType;

use super::{burning_wood, CellRegistry, CellTrait, CellType, Prng};

pub struct DryGrass;
impl DryGrass {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        71
    }
}
impl CellTrait for DryGrass {
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

    fn heatable(&self) -> u8 {
        burning_wood::id()
    }

    fn burnable(&self) -> u8 {
        burning_wood::id()
    }
    fn proton_transfer(&self) -> CellType {
        burning_wood::id()
    }
    fn id(&self) -> CellType {
        71
    }
    fn name(&self) -> &str {
        "dry grass"
    }
}
