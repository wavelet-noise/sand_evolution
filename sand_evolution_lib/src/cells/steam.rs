use super::{helper::fluid_flying_helper, TemperatureContext, *};
use crate::cs::PointType;

pub struct Steam;
impl Steam {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        3
    }
}

impl CellTrait for Steam {
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
        // Steam condenses into water at low temperature
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);

            // Steam condenses into water at temperature below -10 degrees
            // with small probability to avoid instant condensation
            if temperature < 0.0 && prng.next() < 10 {
                use super::water::Water;
                container[cur] = Water::id();
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

    fn name(&self) -> &str {
        "steam"
    }

    fn id(&self) -> CellType {
        3
    }
}
