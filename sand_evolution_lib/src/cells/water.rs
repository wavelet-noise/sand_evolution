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
            // NOTE: Probability depends on temperature only at the boiling point.
            // At T >= 100, evaporate roughly in ~half of ticks.
            if temperature >= 100.0 {
                // 120 / 256 ~= 0.469 (almost half the ticks)
                if dim.next() < 120 {
                    use super::steam::Steam;
                    container[cur] = Steam::id();
                    // Heat is absorbed during evaporation (latent heat): should strongly cool the area,
                    // so fire can't sustain at low temperature.
                    const EVAP_COOLING: f32 = 25.0;
                    (temp_ctx.add_temp)(i, j, -EVAP_COOLING);
                    (temp_ctx.add_temp)(i, j + 1, -EVAP_COOLING);
                    (temp_ctx.add_temp)(i, j - 1, -EVAP_COOLING);
                    (temp_ctx.add_temp)(i + 1, j, -EVAP_COOLING);
                    (temp_ctx.add_temp)(i - 1, j, -EVAP_COOLING);
                    return;
                }
            }
            
            // Water freezes at low temperature
            if temperature < -3.0 {
                // Crystallization releases heat (we only model latent heat on phase change).
                // Warm neighbors a bit to avoid "cold flicker" feedback loops.
                (temp_ctx.add_temp)(i, j + 1, 3.0);
                (temp_ctx.add_temp)(i, j - 1, 3.0);
                (temp_ctx.add_temp)(i + 1, j, 3.0);
                (temp_ctx.add_temp)(i - 1, j, 3.0);

                // Freeze into crushed ice OR snow randomly.
                // NOTE: tweak probabilities here if desired.
                use super::{crushed_ice::CrushedIce, snow::Snow};
                let roll = dim.next();
                container[cur] = if roll < 128 { CrushedIce::id() } else { Snow::id() };
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

    fn shadow_rgba(&self) -> [u8; 4] {
        // Water should cast a softer, slightly bluish shadow (semi-transparent occluder).
        // RGB is a multiplier (255 = no darkening), A controls how strongly it affects shadows.
        [210, 225, 255, 255]
    }

    fn name(&self) -> &str {
        "water"
    }

    fn id(&self) -> CellType {
        2
    }
}
