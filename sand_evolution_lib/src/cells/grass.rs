use crate::cells::coal::Coal;
use crate::cells::dry_grass::DryGrass;
use crate::cs::PointType;

use super::{burning_wood, dry_grass, CellRegistry, CellTrait, CellType, Prng};

pub struct Grass;
impl Grass {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        70
    }
}
impl CellTrait for Grass {
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
        DryGrass::id()
    }

    fn proton_transfer(&self) -> CellType {
        burning_wood::id()
    }
    fn heatable(&self) -> u8 {
        DryGrass::id()
    }
    fn name(&self) -> &str {
        "grass"
    }
    fn id(&self) -> CellType {
        Grass::id()
    }
}
