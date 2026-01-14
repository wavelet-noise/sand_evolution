use crate::cs::PointType;

use super::{water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Ice;
impl Ice {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        55
    }
}
impl CellTrait for Ice {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        if let Some(temp_ctx) = temp_context {
            let temperature = temp_ctx.get_temp(i, j);

            if temperature > 0.0 {
                let chance_f = ((temperature / 20.0) * 26.0).clamp(0.0, 255.0);
                let chance = chance_f as u8;
                if prng.next() < chance {
                    container[cur] = Water::id();
                    const MELT_COOLING: f32 = 5.0;
                    temp_ctx.add_temp(i, j, -MELT_COOLING);
                    temp_ctx.add_temp(i, j + 1, -MELT_COOLING);
                    temp_ctx.add_temp(i, j - 1, -MELT_COOLING);
                    temp_ctx.add_temp(i + 1, j, -MELT_COOLING);
                    temp_ctx.add_temp(i - 1, j, -MELT_COOLING);
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        10
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn heat_proof(&self) -> u8 {
        240
    }
    fn name(&self) -> &str {
        "ice"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
    fn display_color(&self) -> [u8; 3] {
        [77, 153, 255]
    }
}
