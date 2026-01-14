use crate::cs::{self, PointType};

use super::{
    burning_powder::BurningPowder, burning_gas::BurningGas, helper::sand_falling_helper, CellRegistry,
    CellTrait, CellType, Prng, TemperatureContext,
};

pub struct Powder;
impl Powder {
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

impl CellTrait for Powder {
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
        if let Some(temp_ctx) = temp_context {
            let temperature = temp_ctx.get_temp(i, j);

            if temperature >= 300.0 {
                let ignition_chance = if temperature >= 400.0 {
                    180  // ~30% chance at high temperature
                } else {
                    230  // ~10% chance at medium temperature
                };
                if dim.next() > ignition_chance {
                    container[cur] = BurningPowder::id();
                    return;
                }
            }
        }

        sand_falling_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        10
    }
    fn burnable(&self) -> u8 {
        BurningPowder::id()
    }
    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn ignition_temperature(&self) -> Option<f32> {
        Some(300.0)
    }
    fn name(&self) -> &str {
        "powder"
    }
    fn id(&self) -> CellType {
        50
    }
    fn display_color(&self) -> [u8; 3] {
        [26, 26, 26]
    }
}

