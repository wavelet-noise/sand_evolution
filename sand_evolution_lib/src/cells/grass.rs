use crate::cells::dry_grass::DryGrass;
use crate::cs::{self, PointType};

use super::{
    base_water::BaseWater, burning_wood, sand::Base, void::Void, water::Water, CellRegistry,
    CellTrait, CellType, Prng, TemperatureContext,
};

pub struct Grass;
impl Grass {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        70
    }
}
impl CellTrait for Grass {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
        // Check for drying on contact with alkali
        if prng.next() > 180 {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // Check neighboring cells for alkali or alkaline water
            if top_v == Base::id()
                || top_v == BaseWater::id()
                || down_v == Base::id()
                || down_v == BaseWater::id()
                || r_v == Base::id()
                || r_v == BaseWater::id()
                || l_v == Base::id()
                || l_v == BaseWater::id()
            {
                // Grass dries and turns into dry grass
                container[cur] = DryGrass::id();
                return;
            }
        }

        // Slow mixing with dry grass
        if prng.next() > 240 {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // If there's dry grass nearby, green grass slowly turns into dry
            if top_v == DryGrass::id()
                || down_v == DryGrass::id()
                || r_v == DryGrass::id()
                || l_v == DryGrass::id()
            {
                // With very low probability, green grass dries
                if prng.next() > 250 {
                    container[cur] = DryGrass::id();
                    return;
                }
            }
        }

        // Grass grows on neighboring empty cells if there's water near the grass
        // Check rarely for slow growth
        if prng.next() > 200 {
            // First check if there's water near the current grass
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // If there's water near the grass, it can grow
            let has_water_nearby = top_v == Water::id()
                || down_v == Water::id()
                || r_v == Water::id()
                || l_v == Water::id();

            if has_water_nearby {
                // Check neighboring empty cells for growth
                let neighbors = [(top, top_v), (down, down_v), (r, r_v), (l, l_v)];

                // Look for an empty cell for growth
                for (neighbor_idx, neighbor_v) in neighbors.iter() {
                    if *neighbor_v == Void::id() {
                        // With a small probability, grass grows on this empty cell
                        if prng.next() > 220 {
                            container[*neighbor_idx] = Grass::id();
                            return;
                        }
                    }
                }
            }
        }
    }

    fn stat(&self) -> bool {
        true
    }
    fn burnable(&self) -> u8 {
        DryGrass::id()
    }

    fn proton_transfer(&self) -> CellType {
        burning_wood::id()
    }
    fn heatable(&self) -> u8 {
        DryGrass::id()
    }
    fn ignition_temperature(&self) -> Option<f32> {
        Some(400.0)
    }
    fn name(&self) -> &str {
        "grass"
    }
    fn id(&self) -> CellType {
        Grass::id()
    }
    fn display_color(&self) -> [u8; 3] {
        [102, 255, 102]
    }
}
