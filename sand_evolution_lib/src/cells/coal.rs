use crate::cs::PointType;

use super::{
    burning_coal::BurningCoal, burning_gas::BurningGas, helper::sand_falling_helper, CellRegistry,
    CellTrait, CellType, Prng, TemperatureContext,
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
        temp_context: Option<&mut TemperatureContext>,
    ) {
        // Coal can ignite at very high temperature
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Coal ignites at temperature above 150 degrees
            if temperature > 150.0 && dim.next() > 220 {
                container[cur] = BurningCoal::id();
                return;
            }
        }
        
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        10
    }
    fn burnable(&self) -> u8 {
        BurningCoal::id()
    }
    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> &str {
        "coal"
    }
    fn id(&self) -> CellType {
        8
    }
}
