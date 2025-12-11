use crate::cs::PointType;

use super::{{burning_wood, gas::Gas, CellRegistry, CellTrait, CellType, Prng, TemperatureContext}};

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
        // Дерево может загораться при высокой температуре
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Дерево загорается при температуре выше 100 градусов
            if temperature > 100.0 && prng.next() > 200 {
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
    fn name(&self) -> &str {
        "wood"
    }
    fn id(&self) -> CellType {
        50
    }
}
