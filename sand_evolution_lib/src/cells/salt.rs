use crate::cells::molten_salt::MoltenSalt;
use crate::cells::salty_water::SaltyWater;
use crate::cs::PointType;

use super::{
    helper::sand_falling_helper, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
};

pub struct Salt;
impl Salt {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        13
    }
}
impl CellTrait for Salt {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        if let Some(temp_ctx) = temp_context {
            let temperature = temp_ctx.get_temp(i, j);
            const MELT_POINT: f32 = 500.0;
            if temperature > MELT_POINT && prng.next() > 128 {
                let over = (temperature - MELT_POINT) / 200.0;
                let chance = (over * 40.0).clamp(0.0, 255.0) as u8;
                if prng.next() < chance {
                    container[cur] = MoltenSalt::id();
                    temp_ctx.add_temp(i, j, -(temperature - MELT_POINT) * 0.5);
                    return;
                }
            }
        }
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }
    fn den(&self) -> i8 {
        10
    }
    fn dissolve(&self) -> CellType {
        SaltyWater::id()
    }
    fn heatable(&self) -> CellType {
        MoltenSalt::id()
    }
    fn heat_proof(&self) -> u8 {
        200
    }
    fn thermal_conductivity(&self) -> f32 {
        0.7
    }
    fn id(&self) -> CellType {
        13
    }
    fn needs_temp(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "salt"
    }
    fn display_color(&self) -> [u8; 3] {
        [204, 204, 204]
    }
}

