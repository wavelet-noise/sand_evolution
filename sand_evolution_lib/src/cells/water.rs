use super::{*, helper::fluid_falling_helper, TemperatureContext};
use crate::cs::PointType;
pub struct Water;
impl Water {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        2
    }
}
impl CellTrait for Water {
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
        // IMPORTANT: Check temperature BEFORE falling to avoid duplicates
        // If checked after swap, cur no longer contains water!
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Water evaporates into steam at high temperature
            if temperature > 15.0 {
                use super::steam::Steam;
                container[cur] = Steam::id();
                // Heat is absorbed during evaporation (moderately, so adjacent water doesn't freeze)
                (temp_ctx.add_temp)(i, j + 1, -5.0);
                (temp_ctx.add_temp)(i, j - 1, -5.0);
                (temp_ctx.add_temp)(i + 1, j, -5.0);
                (temp_ctx.add_temp)(i - 1, j, -5.0);
                return;
            }
            
            // Water freezes at low temperature
            if temperature < -3.0 {
                use super::crushed_ice::CrushedIce;
                container[cur] = CrushedIce::id();
                return;
            }
        }
        
        // Now try to fall (after temperature check)
        let is_falling = fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1);
        
        if is_falling {
            return;
        }
        
        // When water is not falling, check for dissolution
        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

        let arr = [top, down, l, r];
        let cc = arr[(dim.next() % 4) as usize];

        if dim.next() > 50 {
            let cc_v = container[cc] as usize;
            let cc_c = &pal_container.pal[cc_v];
            let cc_pt = cc_c.dissolve();

            if cc_pt != Void::id() {
                container[cc] = Void::id();
                container[cur] = cc_pt;
                return;
            }
        }
    }

    fn den(&self) -> i8 {
        1
    }

    fn name(&self) -> &str {
        "water"
    }

    fn id(&self) -> CellType {
        2
    }
}
