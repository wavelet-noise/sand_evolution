use super::{{helper::fluid_flying_helper, *, TemperatureContext}};
use crate::cs::PointType;

pub struct Gas;
impl Gas {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        10
    }
}
impl CellTrait for Gas {
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
        // Газ конденсируется в сжиженный газ при очень низкой температуре
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Газ конденсируется при температуре ниже -50 градусов
            // с небольшой вероятностью, чтобы не конденсировался мгновенно
            if temperature < -50.0 && prng.next() < 10 {
                use super::liquid_gas::LiquidGas;
                container[cur] = LiquidGas::id();
                return;
            }
        }
        
        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, prng);
    }

    fn den(&self) -> i8 {
        -1
    }
    fn burnable(&self) -> CellType {
        BurningGas::id()
    }
    fn heatable(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> &str {
        "gas"
    }

    fn id(&self) -> CellType {
        10
    }
}
