use super::{helper::fluid_flying_helper, TemperatureContext, *};
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
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        if let Some(temp_ctx) = temp_context.as_deref() {
            let temperature = (temp_ctx.get_temp)(i, j);

            if temperature >= 150.0 {
                let chance_f = ((temperature - 150.0) * (96.0 / 50.0)).clamp(0.0, 255.0);
                let chance = chance_f as u8;
                if prng.next() < chance {
                    container[cur] = BurningGas::id();
                    return;
                }
            }

            if temperature < -50.0 && prng.next() < 10 {
                use super::liquid_gas::LiquidGas;
                container[cur] = LiquidGas::id();
                return;
            }
        }

        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, prng);
    }

    fn den(&self) -> i8 {
        -1
    }
    fn casts_shadow(&self) -> bool {
        false
    }
    fn burnable(&self) -> CellType {
        Void::id()
    }
    fn heatable(&self) -> CellType {
        Void::id()
    }
    fn name(&self) -> &str {
        "gas"
    }

    fn id(&self) -> CellType {
        10
    }

    fn display_color(&self) -> [u8; 3] {
        [51, 204, 51]
    }
}
