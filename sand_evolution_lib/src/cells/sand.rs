use crate::cells::base_water::BaseWater;
use crate::cells::molten_base::MoltenBase;
use crate::cells::salt::Salt;
use crate::cs::PointType;

use super::{
    gas::Gas, helper::sand_falling_helper, CellRegistry, CellTrait, CellType, Prng,
    TemperatureContext,
};

pub struct Sand;
impl Sand {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        1
    }
}
impl CellTrait for Sand {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }
    fn den(&self) -> i8 {
        10
    }
    fn thermal_conductivity(&self) -> f32 {
        0.5
    }
    fn proton_transfer(&self) -> CellType {
        Gas::id()
    }
    fn display_color(&self) -> [u8; 3] {
        [204, 204, 26]
    }
    fn id(&self) -> CellType {
        1
    }
    fn name(&self) -> &str {
        "sand"
    }
}

pub struct Base;
impl Base {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        14
    }
}
impl CellTrait for Base {
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
            const MELT_POINT: f32 = 400.0;
            if temperature > MELT_POINT && prng.next() > 128 {
                let over = (temperature - MELT_POINT) / 150.0;
                let chance = (over * 50.0).clamp(0.0, 255.0) as u8;
                if prng.next() < chance {
                    container[cur] = MoltenBase::id();
                    temp_ctx.add_temp(i, j, -(temperature - MELT_POINT) * 0.5);
                    return;
                }
            }
        }
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }
    fn den(&self) -> i8 {
        10
    }
    fn proton_transfer(&self) -> CellType {
        Salt::id()
    }
    fn dissolve(&self) -> CellType {
        BaseWater::id()
    }
    fn heatable(&self) -> CellType {
        MoltenBase::id()
    }
    fn heat_proof(&self) -> u8 {
        180
    }
    fn thermal_conductivity(&self) -> f32 {
        0.6
    }
    fn display_color(&self) -> [u8; 3] {
        [190, 20, 45]
    }
    fn id(&self) -> CellType {
        14
    }
    fn needs_temp(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "base"
    }
}
