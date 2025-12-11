use crate::cs::{self, PointType};

use super::{
    void::Void, water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
};

pub struct CrushedIce;
impl CrushedIce {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        56
    }
}
impl CellTrait for CrushedIce {
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
        // Crushed ice melts only based on temperature - if temperature > 0, it melts and cools the environment
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // If temperature is above 0 degrees, crushed ice melts
            // Check rarely to avoid fast melt/freeze cycles
            if prng.next() > 200 {
                if temperature > 0.0 {
                    // Give cold to neighbors when melting (reduced to avoid flashes)
                    (temp_ctx.add_temp)(i, j + 1, -3.0); // top
                    (temp_ctx.add_temp)(i, j - 1, -3.0); // bottom
                    (temp_ctx.add_temp)(i + 1, j, -3.0); // left
                    (temp_ctx.add_temp)(i - 1, j, -3.0); // right
                    
                    container[cur] = Water::id();
                    return;
                }
            }
        }
        
        // Special logic for crushed ice: floats on water (density 0), but falls through Void
        // Check cell below
        let down = cs::xy_to_index(i, j - 1);
        let down_v = container[down] as usize;
        let down_c = &pal_container.pal[down_v];
        
        // If Void below, fall down
        if down_v == Void::id() as usize {
            container.swap(cur, down);
            return;
        }
        
        // If something with density less than 0 below (lighter than crushed ice), fall
        if down_c.den() < self.den() && !down_c.stat() {
            container.swap(cur, down);
            return;
        }
        
        // Check diagonals down
        const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
        let selected_order = ORDER[(prng.next() % 2) as usize];
        
        for k in 0..2 {
            match selected_order[k] {
                0 => {
                    let dr = cs::xy_to_index(i + 1, j - 1);
                    let dr_v = container[dr] as usize;
                    let dr_c = &pal_container.pal[dr_v];
                    if dr_v == Void::id() as usize || (dr_c.den() < self.den() && !dr_c.stat()) {
                        container.swap(cur, dr);
                        return;
                    }
                }
                1 => {
                    let dl = cs::xy_to_index(i - 1, j - 1);
                    let dl_v = container[dl] as usize;
                    let dl_c = &pal_container.pal[dl_v];
                    if dl_v == Void::id() as usize || (dl_c.den() < self.den() && !dl_c.stat()) {
                        container.swap(cur, dl);
                        return;
                    }
                }
                _ => (),
            }
        }

        if prng.next() > 1 {
            return;
        }

        if prng.next() > 32 {
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let bot = cs::xy_to_index(i, j - 1);
        let left = cs::xy_to_index(i + 1, j);
        let right = cs::xy_to_index(i - 1, j);

        // Crushed ice should not turn into water on contact
        // Only melting at temperature > 0
    }

    fn den(&self) -> i8 {
        0
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn heat_proof(&self) -> u8 {
        200
    }
    fn name(&self) -> &str {
        "crushed ice"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
