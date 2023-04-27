use crate::cs::PointType;

use super::{CellType, CellTrait, CellRegistry, Dim, helper::sand_faling_helper, burning_coal};

pub struct Coal;
impl Coal {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        8
    }
}
impl CellTrait for Coal {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Dim,
    ) {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        2
    }
    fn burnable(&self) -> u8 {
        burning_coal::id()
    }
    fn id(&self) -> CellType {
        8
    }
    fn name(&self) -> String {
        "coal".to_owned()
    }
}