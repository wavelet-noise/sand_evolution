use crate::cells::acid::Acid;
use crate::cells::helper::fluid_falling_helper;
use crate::cells::steam::Steam;
use crate::cells::void::Void;
use crate::cells::water::Water;
use crate::cells::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};
use crate::cs;
use crate::cs::PointType;

pub struct DeluteAcid;

impl DeluteAcid {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        83
    }
}

impl CellTrait for DeluteAcid {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
        mut temp_context: Option<&mut TemperatureContext>,
    ) {
        if let Some(ref mut temp_ctx) = temp_context {
            let temperature = temp_ctx.get_temp(i, j);

            if temperature >= 100.0 {
                if dim.next() < 100 {
                    if dim.next() < 128 {
                        container[cur] = Steam::id();
                    } else {
                        container[cur] = Acid::id();
                    }
                    const EVAP_COOLING: f32 = 45.0;
                    temp_ctx.add_temp(i, j, -EVAP_COOLING);
                    temp_ctx.add_temp(i, j + 1, -EVAP_COOLING * 0.5);
                    temp_ctx.add_temp(i, j - 1, -EVAP_COOLING * 0.5);
                    temp_ctx.add_temp(i + 1, j, -EVAP_COOLING * 0.5);
                    temp_ctx.add_temp(i - 1, j, -EVAP_COOLING * 0.5);
                    return;
                }
            }
        }

        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1) {
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
                    if let Some(ref mut temp_ctx) = temp_context {
                        temp_ctx.add_temp(i, j + 1, 10.0);
                        temp_ctx.add_temp(i, j - 1, 10.0);
                        temp_ctx.add_temp(i + 1, j, 10.0);
                        temp_ctx.add_temp(i - 1, j, 10.0);
                        temp_ctx.add_temp(i, j, 5.0);
                    }
                    return;
                }
            }

            if dim.next() > 240 {
                let cc_v = container[cc] as usize;
                let cc_c = &pal_container.pal[cc_v];
                let cc_pt = cc_c.proton_transfer();

                if cc_pt != Void::id() {
                    container[cc] = cc_pt;

                    if dim.next() > 120 {
                        container[cur] = Water::id();
                    } else {
                        container[cur] = Void::id();
                    }
                    if let Some(ref mut temp_ctx) = temp_context {
                        temp_ctx.add_temp(i, j + 1, 8.0);
                        temp_ctx.add_temp(i, j - 1, 8.0);
                        temp_ctx.add_temp(i + 1, j, 8.0);
                        temp_ctx.add_temp(i - 1, j, 8.0);
                        temp_ctx.add_temp(i, j, 4.0);
                    }
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        2
    }

    fn needs_temp(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "delute acid"
    }

    fn id(&self) -> CellType {
        83
    }
}
