use super::{
    helper::{fluid_falling_helper, fluid_flying_helper},
    *,
};
use crate::cs::PointType;

pub struct Gas;
impl Gas {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        10
    }
}
impl CellTrait for Gas {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        -1
    }
    fn burnable(&self) -> CellType {
        BurningGas::id()
    }
    fn heatable(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> String {
        "gas".to_owned()
    }

    fn id(&self) -> CellType {
        10
    }
}