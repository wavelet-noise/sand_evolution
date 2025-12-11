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
        _prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        // Ice melts only based on temperature - if temperature > 0, it melts and cools the environment
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // If temperature is above 0 degrees, ice melts
            if temperature > 0.0 {
                container[cur] = Water::id();
                return;
            }
        }

        // Ice should not turn into water on contact with void or water
        // Only melting at temperature > 0
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
}
