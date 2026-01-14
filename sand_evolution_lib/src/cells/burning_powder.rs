use super::{helper::try_spawn_smoke, TemperatureContext, *};
use crate::cs::{self, PointType};

pub struct BurningPowder;
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
        if !sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng) {
            if let Some(temp_ctx) = temp_context.as_deref_mut() {
                temp_ctx.add_temp(i, j, 400.0);
                
                temp_ctx.add_temp(i, j + 1, 300.0);
                temp_ctx.add_temp(i, j - 1, 300.0);
                temp_ctx.add_temp(i + 1, j, 300.0);
                temp_ctx.add_temp(i - 1, j, 300.0);
                
                if i > 0 && j > 0 {
                    temp_ctx.add_temp(i - 1, j - 1, 200.0);
                }
                if i > 0 && j + 1 < cs::SECTOR_SIZE.y {
                    temp_ctx.add_temp(i - 1, j + 1, 200.0);
                }
                if i + 1 < cs::SECTOR_SIZE.x && j > 0 {
                    temp_ctx.add_temp(i + 1, j - 1, 200.0);
                }
                if i + 1 < cs::SECTOR_SIZE.x && j + 1 < cs::SECTOR_SIZE.y {
                    temp_ctx.add_temp(i + 1, j + 1, 200.0);
                }
                
                if i > 1 {
                    temp_ctx.add_temp(i - 2, j, 150.0);
                }
                if i + 2 < cs::SECTOR_SIZE.x {
                    temp_ctx.add_temp(i + 2, j, 150.0);
                }
                if j > 1 {
                    temp_ctx.add_temp(i, j - 2, 150.0);
                }
                if j + 2 < cs::SECTOR_SIZE.y {
                    temp_ctx.add_temp(i, j + 2, 150.0);
                }
            }
            let bot = cs::xy_to_index(i, j - 1);
            let bot_v = container[bot] as usize;

            let top = cs::xy_to_index(i, j + 1);

            if prng.next() > 60 {
                let smoke_count = if prng.next() > 100 { 
                    if prng.next() > 150 { 4 } else { 3 }
                } else { 
                    2 
                };
                try_spawn_smoke(i, j, container, prng, smoke_count);
            }

            if prng.next() > 150 {
                return;
            }

            if container[top] == Water::id() {
                container[top] = Steam::id();
                return;
            }

            if prng.next() > 120 {
                if let Some(temp_ctx) = temp_context.as_deref_mut() {
                    temp_ctx.add_temp(i, j, 500.0);
                    temp_ctx.add_temp(i, j + 1, 500.0);
                    temp_ctx.add_temp(i, j - 1, 500.0);
                    temp_ctx.add_temp(i + 1, j, 500.0);
                    temp_ctx.add_temp(i - 1, j, 500.0);
                    if i > 0 && j > 0 {
                        temp_ctx.add_temp(i - 1, j - 1, 350.0);
                    }
                    if i > 0 && j + 1 < cs::SECTOR_SIZE.y {
                        temp_ctx.add_temp(i - 1, j + 1, 350.0);
                    }
                    if i + 1 < cs::SECTOR_SIZE.x && j > 0 {
                        temp_ctx.add_temp(i + 1, j - 1, 350.0);
                    }
                    if i + 1 < cs::SECTOR_SIZE.x && j + 1 < cs::SECTOR_SIZE.y {
                        temp_ctx.add_temp(i + 1, j + 1, 350.0);
                    }
                    if i > 1 {
                        temp_ctx.add_temp(i - 2, j, 250.0);
                    }
                    if i + 2 < cs::SECTOR_SIZE.x {
                        temp_ctx.add_temp(i + 2, j, 250.0);
                    }
                    if j > 1 {
                        temp_ctx.add_temp(i, j - 2, 250.0);
                    }
                    if j + 2 < cs::SECTOR_SIZE.y {
                        temp_ctx.add_temp(i, j + 2, 250.0);
                    }
                }
                try_spawn_smoke(i, j, container, prng, 6);
                container[cur] = Void::id();
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

