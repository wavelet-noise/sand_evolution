use crate::cs::PointType;

use super::{CellType, CellTrait, CellRegistry, Dim};

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
        _: &mut Dim,
    ) {
    }

    fn stat(&self) -> bool {
        true
    }

    fn id(&self) -> CellType {
        255
    }
    fn name(&self) -> String {
        "stone".to_owned()
    }
}