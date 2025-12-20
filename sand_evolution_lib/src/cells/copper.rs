use crate::cs::PointType;

use super::{CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

/// Copper — static solid with very high thermal conductivity.
pub struct Copper;

impl Copper {
    pub const fn new() -> Self {
        Self
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    /// Chosen free id in low range.
    pub fn id() -> CellType {
        20
    }
}

impl CellTrait for Copper {
    fn update(
        &self,
        _i: PointType,
        _j: PointType,
        _cur: usize,
        _container: &mut [CellType],
        _pal_container: &CellRegistry,
        _prng: &mut Prng,
        _temp_context: Option<&mut TemperatureContext>,
    ) {
        // Static, no local behavior. Heat transport handled by global diffusion.
    }

    fn den(&self) -> i8 {
        20
    }

    fn stat(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "copper"
    }

    fn id(&self) -> CellType {
        Self::id()
    }

    /// High thermal conductivity: copper quickly equalizes temperature.
    fn thermal_conductivity(&self) -> f32 {
        2.5
    }

    /// Warm copper-like shadow tint (slightly reddish, not too bright).
    fn shadow_rgba(&self) -> [u8; 4] {
        [200, 150, 120, 255]
    }

    fn display_color(&self) -> [u8; 3] {
        [184, 115, 51]
    }
}


