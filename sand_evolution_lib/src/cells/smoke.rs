use super::{helper::fluid_flying_helper, TemperatureContext, *};
use crate::cs::PointType;

pub struct Smoke;
impl Smoke {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        21
    }
}

impl CellTrait for Smoke {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        _temp_context: Option<&mut TemperatureContext>,
    ) {
        if prng.next() < 2 {
            container[cur] = Void::id();
            return;
        }

        // Smoke rises slowly upward, like snow but upward
        // Use probability checks similar to snow to make it move slowly
        if prng.next() > 128 {
            // 50% chance to try moving
            if fluid_flying_helper(self.den(), i, j, container, pal_container, cur, prng) {
                return;
            }
        }

        // Additional slow movement check
        if prng.next() > 50 {
            return;
        }

        // Try moving upward
        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, prng);
    }

    fn den(&self) -> i8 {
        -1
    }

    fn casts_shadow(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "smoke"
    }

    fn id(&self) -> CellType {
        Self::id()
    }

    fn display_color(&self) -> [u8; 3] {
        [0, 0, 0] // Black
    }
}

