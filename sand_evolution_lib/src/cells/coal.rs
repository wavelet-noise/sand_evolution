use crate::cs::PointType;
use crate::cs;

use super::{
    burning_coal::BurningCoal, burning_gas::BurningGas, helper::sand_falling_helper, CellRegistry,
    CellTrait, CellType, Prng, TemperatureContext, void::Void,
};

pub struct Coal;
impl Coal {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        8
    }
}

fn has_adjacent_air(i: PointType, j: PointType, container: &[CellType]) -> bool {
    if i > 0 && container[cs::xy_to_index(i - 1, j)] == Void::id() {
        return true;
    }
    if i + 1 < cs::SECTOR_SIZE.x && container[cs::xy_to_index(i + 1, j)] == Void::id() {
        return true;
    }
    if j > 0 && container[cs::xy_to_index(i, j - 1)] == Void::id() {
        return true;
    }
    if j + 1 < cs::SECTOR_SIZE.y && container[cs::xy_to_index(i, j + 1)] == Void::id() {
        return true;
    }
    false
}

impl CellTrait for Coal {
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
        if let Some(temp_ctx) = temp_context {
            let temperature = temp_ctx.get_temp(i, j);

            if temperature >= 450.0 && dim.next() > 235 && has_adjacent_air(i, j, container) {
                container[cur] = BurningCoal::id();
                return;
            }
        }

        sand_falling_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        10
    }
    fn burnable(&self) -> u8 {
        BurningCoal::id()
    }
    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn ignition_temperature(&self) -> Option<f32> {
        Some(300.0)
    }
    fn name(&self) -> &str {
        "coal"
    }
    fn id(&self) -> CellType {
        8
    }
    fn display_color(&self) -> [u8; 3] {
        [26, 26, 26]
    }
}
