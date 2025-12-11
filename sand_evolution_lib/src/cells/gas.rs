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
        if let Some(temp_ctx) = temp_context.as_deref() {
            let temperature = (temp_ctx.get_temp)(i, j);

            // Gas should ignite well from temperature.
            // We model this as a probability that grows with temperature once above a threshold.
            //
            // Start igniting from "hot air" temps; at 200°C this is already quite likely.
            // p(T) = clamp((T - 150) * 1.92, 0..255) / 256
            // Examples:
            // - 150°C => 0%
            // - 180°C => ~22%
            // - 200°C => ~37.5%
            // - 250°C => ~75%
            // - 300°C => ~99%
            if temperature >= 150.0 {
                let chance_f = ((temperature - 150.0) * (96.0 / 50.0)).clamp(0.0, 255.0);
                let chance = chance_f as u8;
                if prng.next() < chance {
                    container[cur] = BurningGas::id();
                    return;
                }
            }

            // Gas condenses into liquid gas at very low temperature.
            // With small probability to avoid instant condensation.
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
        // Disable "contact ignition" routes (neighbor rules that use burnable()).
        // Gas should ignite via temperature logic instead.
        Void::id()
    }
    fn heatable(&self) -> CellType {
        // Disable "contact ignition" routes (neighbor rules that use heatable()).
        // Gas should ignite via temperature logic instead.
        Void::id()
    }
    fn name(&self) -> &str {
        "gas"
    }

    fn id(&self) -> CellType {
        10
    }
}
