use super::{TemperatureContext, *};
use crate::cs::{self, PointType};

pub struct BurningPowder;

// Helper function to spawn smoke particles in nearby Void cells
fn try_spawn_smoke(
    i: PointType,
    j: PointType,
    container: &mut [CellType],
    prng: &mut Prng,
    target_count: usize,
) -> usize {
    let mut spawned = 0;
    let mut candidates = Vec::new();

    // Collect all adjacent Void cells as candidates
    if j + 1 < cs::SECTOR_SIZE.y {
        let top = cs::xy_to_index(i, j + 1);
        if container[top] == Void::id() {
            candidates.push(top);
        }
    }
    if j > 0 {
        let bot = cs::xy_to_index(i, j - 1);
        if container[bot] == Void::id() {
            candidates.push(bot);
        }
    }
    if i + 1 < cs::SECTOR_SIZE.x {
        let right = cs::xy_to_index(i + 1, j);
        if container[right] == Void::id() {
            candidates.push(right);
        }
    }
    if i > 0 {
        let left = cs::xy_to_index(i - 1, j);
        if container[left] == Void::id() {
            candidates.push(left);
        }
    }

    // Shuffle candidates using prng and spawn smoke
    while spawned < target_count && !candidates.is_empty() {
        let idx = (prng.next() as usize) % candidates.len();
        let cell_idx = candidates.remove(idx);
        container[cell_idx] = Smoke::id();
        spawned += 1;
    }

    spawned
}
impl BurningPowder {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn id() -> CellType {
        51
    }
}

impl CellTrait for BurningPowder {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        mut temp_context: Option<&mut TemperatureContext>,
    ) {
        const BURNING_POWDER_SUSTAIN_TEMP: f32 = 90.0;
        let mut extinguish = false;
        if let Some(temp_ctx) = temp_context.as_deref() {
            let mut sum = (temp_ctx.get_temp)(i, j);
            let mut n = 1.0f32;

            if i > 0 {
                sum += (temp_ctx.get_temp)(i - 1, j);
                n += 1.0;
            }
            if i + 1 < cs::SECTOR_SIZE.x {
                sum += (temp_ctx.get_temp)(i + 1, j);
                n += 1.0;
            }
            if j > 0 {
                sum += (temp_ctx.get_temp)(i, j - 1);
                n += 1.0;
            }
            if j + 1 < cs::SECTOR_SIZE.y {
                sum += (temp_ctx.get_temp)(i, j + 1);
                n += 1.0;
            }

            extinguish = (sum / n) < BURNING_POWDER_SUSTAIN_TEMP;
        }

        // Burning powder releases heat
        if let Some(temp_ctx) = temp_context.as_deref_mut() {
            (temp_ctx.add_temp)(i, j + 1, 3.0); // top
            (temp_ctx.add_temp)(i, j - 1, 3.0); // bottom
            (temp_ctx.add_temp)(i + 1, j, 3.0); // right
            (temp_ctx.add_temp)(i - 1, j, 3.0); // left
        }
        if extinguish {
            // Try to spawn 4 smoke particles when extinguishing
            try_spawn_smoke(i, j, container, prng, 4);
            container[cur] = Powder::id();
            return;
        }
        if !sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng) {
            let bot = cs::xy_to_index(i, j - 1);
            let bot_v = container[bot] as usize;

            let top = cs::xy_to_index(i, j + 1);

            // Generate smoke continuously during burning
            if prng.next() > 100 {
                // ~60% chance to spawn smoke each tick
                try_spawn_smoke(i, j, container, prng, 1);
            }

            if prng.next() > 200 {
                return;
            }

            if container[top] == Water::id() {
                container[top] = Steam::id();
                if prng.next() > 200 {
                    container[cur] = Powder::id();
                }
                return;
            }

            if prng.next() > 250 {
                // Try to spawn 4 smoke particles when despawning
                try_spawn_smoke(i, j, container, prng, 4);
                container[cur] = Void::id();
                //prng.add_carb();
                return;
            }

            let topl = cs::xy_to_index(i - 1, j + 1);
            let topr = cs::xy_to_index(i + 1, j + 1);

            let arr = [top, topl, topr];
            let cc = arr[(prng.next() % 3) as usize];
            let top_v = container[cc];

            if top_v == Void::id() {
                container[cc] = fire::id();
            }

            if prng.next() > 50 {
                return;
            }

            let bot_c = &pal_container.pal[bot_v];
            let bot_b = bot_c.burnable();

            if bot_b != Void::id() {
                container[bot] = bot_b;
                return;
            }
        }
    }

    fn den(&self) -> i8 {
        2
    }
    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> &str {
        "burning powder"
    }

    fn id(&self) -> CellType {
        51
    }
}

