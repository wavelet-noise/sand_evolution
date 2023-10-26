use crate::cs::PointType;

use super::{
    burning_coal::{BurningCoal},
    helper::sand_faling_helper,
    CellRegistry, CellTrait, CellType, Prng, burning_gas::BurningGas,
};

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
        dim: &mut Prng,
    ) {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        10
    }
    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn burnable(&self) -> u8 {
        BurningCoal::id()
    }
    fn id(&self) -> CellType {
        8
    }
    fn name(&self) -> &str {
        "coal"
    }
}
