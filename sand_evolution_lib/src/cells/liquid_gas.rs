use super::{*, helper::fluid_falling_helper, TemperatureContext};
use crate::cs::PointType;

pub struct LiquidGas;
impl LiquidGas {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        17
    }
}

impl CellTrait for LiquidGas {
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
        // Проверяем температуру ПЕРЕД падением
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Сжиженный газ испаряется обратно в газ при температуре выше -30
            // с вероятностью, чтобы не испарялся мгновенно
            if temperature > -4.0 && prng.next() < 30 {
                use super::gas::Gas;
                container[cur] = Gas::id();
                // При испарении поглощается тепло
                (temp_ctx.add_temp)(i, j + 1, -3.0);
                (temp_ctx.add_temp)(i, j - 1, -3.0);
                (temp_ctx.add_temp)(i + 1, j, -3.0);
                (temp_ctx.add_temp)(i - 1, j, -3.0);
                return;
            }
        }
        
        // Сжиженный газ ведет себя как жидкость
        fluid_falling_helper(self.den(), i, j, container, pal_container, cur, prng, 1);
    }

    fn den(&self) -> i8 {
        // Легче воды, но всё же жидкость
        1
    }

    fn burnable(&self) -> CellType {
        // Сжиженный газ тоже горючий
        BurningGas::id()
    }

    fn heatable(&self) -> CellType {
        // При нагреве становится газом (а затем может загореться)
        Gas::id()
    }

    fn name(&self) -> &str {
        "liquid_gas"
    }

    fn id(&self) -> CellType {
        17
    }
}
