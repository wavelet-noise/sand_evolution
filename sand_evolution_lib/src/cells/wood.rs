use crate::cs::PointType;

use super::{burning_wood, CellRegistry, CellTrait, CellType, Dim};

pub struct Wood;
impl Wood {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        5
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
        _: &mut Dim,
    ) {
    }

    fn stat(&self) -> bool {
        true
    }
    fn burnable(&self) -> u8 {
        burning_wood::id()
    }
    fn id(&self) -> CellType {
        5
    }
    fn name(&self) -> String {
        "wood".to_owned()
    }
}
