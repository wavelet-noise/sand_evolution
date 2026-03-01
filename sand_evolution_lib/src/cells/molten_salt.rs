use crate::cells::helper::fluid_falling_helper;
use crate::cells::salt::Salt;
use crate::cells::salty_water::SaltyWater;
use crate::cells::void::Void;
use crate::cells::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};
use crate::cs::PointType;

pub struct MoltenSalt;

impl MoltenSalt {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        86
    }
}

impl CellTrait for MoltenSalt {
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

            const TARGET_TEMP: f32 = 400.0;
            let deficit = TARGET_TEMP - temperature;
            if deficit > 0.0 {
                let heat = deficit * 0.4;
                temp_ctx.add_temp(i, j, heat);

                let crystal_chance = ((deficit / TARGET_TEMP) * 30.0).clamp(0.0, 255.0) as u8;
                if prng.next() < crystal_chance {
                    container[cur] = Salt::id();
                    return;
                }
            }
        }

        fluid_falling_helper(self.den(), i, j, container, pal_container, cur, prng, 2);
    }

    fn den(&self) -> i8 {
        4
    }

    fn dissolve(&self) -> CellType {
        SaltyWater::id()
    }

    fn shadow_rgba(&self) -> [u8; 4] {
        [255, 180, 100, 200]
    }

    fn thermal_conductivity(&self) -> f32 {
        1.2
    }

    fn convection_factor(&self) -> f32 {
        0.8
    }

    fn heatable(&self) -> CellType {
        Void::id()
    }

    fn needs_temp(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "molten salt"
    }

    fn id(&self) -> CellType {
        86
    }

    fn display_color(&self) -> [u8; 3] {
        [255, 140, 50]
    }
}
