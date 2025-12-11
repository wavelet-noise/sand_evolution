use crate::cs::{self, PointType};

use super::{
    helper::fluid_falling_helper, void::Void, water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
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
        // Snow melts only based on temperature - if temperature > 0, it melts and cools the environment
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // If temperature is above 0 degrees, snow melts
            if temperature > 0.0 {
                // Give cold to neighbors when melting (reduced to avoid flashes)
                (temp_ctx.add_temp)(i, j + 1, -5.0); // top
                (temp_ctx.add_temp)(i, j - 1, -5.0); // bottom
                (temp_ctx.add_temp)(i + 1, j, -5.0); // left
                (temp_ctx.add_temp)(i - 1, j, -5.0); // right
                
                container[cur] = Water::id();
                return;
            }
        }
        
        if prng.next() > 128 && fluid_falling_helper(self.den(), i, j, container, pal_container, cur, prng, 10)
        {
            return;
        }

        if prng.next() > 1 {
            return;
        }

        if prng.next() > 50 {
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let bot = cs::xy_to_index(i, j - 1);
        let left = cs::xy_to_index(i + 1, j);
        let right = cs::xy_to_index(i - 1, j);

        // Snow should not turn into water on contact
        // Only melting at temperature > 0 (if temperature system is added)
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
}
