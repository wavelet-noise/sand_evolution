use crate::cs::PointType;

use crate::cs;

use super::{{burning_wood, gas::Gas, void::Void, CellRegistry, CellTrait, CellType, Prng, TemperatureContext}};

pub struct Wood;
impl Wood {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        50
    }
}

fn has_adjacent_air(i: PointType, j: PointType, container: &[CellType]) -> bool {
    // "Access to air": at least one orthogonal neighbor is Void.
    // Bounds checks avoid u16 underflow/wrap at edges.
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
impl CellTrait for Wood {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        // Wood can auto-ignite only when very hot (environment ignition).
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Keep auto-ignition high, and make it rare: otherwise it behaves like gunpowder.
            if temperature >= 320.0 && prng.next() > 200 && has_adjacent_air(i, j, container) {
                container[cur] = burning_wood::id();
                return;
            }
        }
    }

    fn stat(&self) -> bool {
        true
    }
    fn burnable(&self) -> u8 {
        burning_wood::id()
    }
    fn proton_transfer(&self) -> CellType {
        Gas::id()
    }
    fn ignition_temperature(&self) -> Option<f32> {
        // "Can ignite from external fire" threshold.
        // Actual ignition probability is handled in fire.rs and scales with (temp - threshold),
        // so at low heat it will ignite very slowly.
        Some(260.0)
    }
    fn name(&self) -> &str {
        "wood"
    }
    fn id(&self) -> CellType {
        50
    }
}
