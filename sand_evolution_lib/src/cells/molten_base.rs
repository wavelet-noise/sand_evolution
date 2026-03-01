use crate::cells::base_water::BaseWater;
use crate::cells::helper::fluid_falling_helper;
use crate::cells::sand::Base;
use crate::cells::salt::Salt;
use crate::cells::void::Void;
use crate::cells::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};
use crate::cs;
use crate::cs::PointType;

pub struct MoltenBase;

impl MoltenBase {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        87
    }
}

impl CellTrait for MoltenBase {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        mut temp_context: Option<&mut TemperatureContext>,
    ) {
        if let Some(ref mut temp_ctx) = temp_context {
            let temperature = temp_ctx.get_temp(i, j);

            const TARGET_TEMP: f32 = 300.0;
            let deficit = TARGET_TEMP - temperature;
            if deficit > 0.0 {
                let heat = deficit * 0.35;
                temp_ctx.add_temp(i, j, heat);

                let crystal_chance = ((deficit / TARGET_TEMP) * 35.0).clamp(0.0, 255.0) as u8;
                if prng.next() < crystal_chance {
                    container[cur] = Base::id();
                    return;
                }
            }
        }

        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, prng, 2) {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let arr = [top, down, l, r];
            let cc = arr[(prng.next() % 4) as usize];

            if prng.next() > 30 {
                let cc_v = container[cc] as usize;
                let cc_c = &pal_container.pal[cc_v];
                let cc_pt = cc_c.proton_transfer();

                if cc_pt != Void::id()
                    && cc_v != MoltenBase::id() as usize
                    && cc_v != Base::id() as usize
                {
                    container[cc] = cc_pt;
                    if let Some(ref mut temp_ctx) = temp_context {
                        temp_ctx.add_temp(i, j + 1, 15.0);
                        temp_ctx.add_temp(i, j - 1, 15.0);
                        temp_ctx.add_temp(i + 1, j, 15.0);
                        temp_ctx.add_temp(i - 1, j, 15.0);
                        temp_ctx.add_temp(i, j, 10.0);
                    }
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        4
    }

    fn proton_transfer(&self) -> CellType {
        Salt::id()
    }

    fn dissolve(&self) -> CellType {
        BaseWater::id()
    }

    fn shadow_rgba(&self) -> [u8; 4] {
        [255, 120, 100, 200]
    }

    fn thermal_conductivity(&self) -> f32 {
        1.0
    }

    fn convection_factor(&self) -> f32 {
        0.8
    }

    fn heatable(&self) -> CellType {
        Void::id()
    }

    fn needs_temp(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "molten base"
    }

    fn id(&self) -> CellType {
        87
    }

    fn display_color(&self) -> [u8; 3] {
        [220, 50, 180]
    }
}
