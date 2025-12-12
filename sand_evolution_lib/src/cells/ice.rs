use crate::cs::PointType;

use super::{water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Ice;
impl Ice {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        55
    }
}
impl CellTrait for Ice {
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
        // Ice melts only based on temperature.
        // Important: make it probabilistic to avoid instant melting at mild temperatures (e.g. +20°C).
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);

            // If temperature is above 0 degrees, ice can melt with probability depending on temperature.
            // Design target: at +20°C => ~10% chance per tick.
            // We scale linearly: p(20)=26/256≈10.16%; p grows with temperature and clamps to [0..255].
            if temperature > 0.0 {
                let chance_f = ((temperature / 20.0) * 26.0).clamp(0.0, 255.0);
                let chance = chance_f as u8;
                if prng.next() < chance {
                    container[cur] = Water::id();
                    // Latent heat of fusion: melting consumes heat, so the local area should cool down.
                    // Calibrated relative to water evaporation cooling (see `water.rs`).
                    const MELT_COOLING: f32 = 5.0;
                    (temp_ctx.add_temp)(i, j, -MELT_COOLING);
                    (temp_ctx.add_temp)(i, j + 1, -MELT_COOLING);
                    (temp_ctx.add_temp)(i, j - 1, -MELT_COOLING);
                    (temp_ctx.add_temp)(i + 1, j, -MELT_COOLING);
                    (temp_ctx.add_temp)(i - 1, j, -MELT_COOLING);
                    return;
                }
            }
        }

        // Ice should not turn into water on contact with void or water
        // Only melting at temperature > 0
    }

    fn den(&self) -> i8 {
        10
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn heat_proof(&self) -> u8 {
        240
    }
    fn name(&self) -> &str {
        "ice"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
