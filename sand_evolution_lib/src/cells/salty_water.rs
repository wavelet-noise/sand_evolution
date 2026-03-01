use crate::cells::helper::fluid_falling_helper;
use crate::cells::salt::Salt;
use crate::cells::steam::Steam;
use crate::cells::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};
use crate::cs::PointType;

pub struct SaltyWater;

impl SaltyWater {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        84
    }
}

impl CellTrait for SaltyWater {
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

            if temperature >= 105.0 {
                if dim.next() < 100 {
                    if dim.next() < 128 {
                        container[cur] = Steam::id();
                    } else {
                        container[cur] = Salt::id();
                    }
                    const EVAP_COOLING: f32 = 50.0;
                    temp_ctx.add_temp(i, j, -EVAP_COOLING);
                    temp_ctx.add_temp(i, j + 1, -EVAP_COOLING * 0.5);
                    temp_ctx.add_temp(i, j - 1, -EVAP_COOLING * 0.5);
                    temp_ctx.add_temp(i + 1, j, -EVAP_COOLING * 0.5);
                    temp_ctx.add_temp(i - 1, j, -EVAP_COOLING * 0.5);
                    return;
                }
            }
        }

        fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1);
    }

    fn den(&self) -> i8 {
        2
    }

    fn shadow_rgba(&self) -> [u8; 4] {
        [205, 220, 255, 115]
    }

    fn needs_temp(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "salty water"
    }

    fn id(&self) -> CellType {
        84
    }
    fn display_color(&self) -> [u8; 3] {
        [128, 128, 255]
    }
}
