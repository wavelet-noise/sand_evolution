use super::{helper::fluid_flying_helper, TemperatureContext, *};
use crate::cs::PointType;

pub struct CompressedSteam;
impl CompressedSteam {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        81
    }
}

impl CellTrait for CompressedSteam {
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
        // Will split into 2 uncompressed steam cells
        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, prng);
    }

    fn den(&self) -> i8 {
        -1
    }
    fn casts_shadow(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "compressed_steam"
    }

    fn id(&self) -> CellType {
        81
    }

    fn display_color(&self) -> [u8; 3] {
        [128, 128, 128]
    }
}

