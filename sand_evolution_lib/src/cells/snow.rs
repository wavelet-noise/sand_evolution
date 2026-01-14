use crate::cs::{self, PointType};

use super::{
    helper::fluid_falling_helper, water::Water, CellRegistry, CellTrait, CellType, Prng,
    TemperatureContext,
};

pub struct Snow;
impl Snow {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        57
    }
}
impl CellTrait for Snow {
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

            if temperature > 0.0 {
                container[cur] = Water::id();
                const MELT_COOLING: f32 = 2.0;
                temp_ctx.add_temp(i, j, -MELT_COOLING);
                temp_ctx.add_temp(i, j + 1, -MELT_COOLING);
                temp_ctx.add_temp(i, j - 1, -MELT_COOLING);
                temp_ctx.add_temp(i + 1, j, -MELT_COOLING);
                temp_ctx.add_temp(i - 1, j, -MELT_COOLING);
                return;
            }
        }

        if prng.next() > 128
            && fluid_falling_helper(self.den(), i, j, container, pal_container, cur, prng, 10)
        {
            return;
        }

        if prng.next() > 1 {
            return;
        }

        if prng.next() > 50 {
            return;
        }

        let _top = cs::xy_to_index(i, j + 1);
        let _bot = cs::xy_to_index(i, j - 1);
        let _left = cs::xy_to_index(i + 1, j);
        let _right = cs::xy_to_index(i - 1, j);
    }

    fn den(&self) -> i8 {
        5
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }
    fn name(&self) -> &str {
        "snow"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
    fn display_color(&self) -> [u8; 3] {
        [204, 230, 255]
    }
}
