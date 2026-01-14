use super::{helper::try_spawn_smoke, TemperatureContext, *};
use crate::cs::{self, PointType};

pub const fn new() -> Cell {
    Cell
}
pub fn boxed() -> Box<Cell> {
    Box::new(new())
}

pub fn id() -> CellType {
    6
}

pub struct Cell;
impl CellTrait for Cell {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        mut temp_context: Option<&mut TemperatureContext>,
    ) {
        const BURNING_WOOD_SUSTAIN_TEMP: f32 = 60.0;
        let mut extinguish = false;
        if let Some(temp_ctx) = temp_context.as_deref() {
            let mut sum = temp_ctx.get_temp(i, j);
            let mut n = 1.0f32;

            if i > 0 {
                sum += temp_ctx.get_temp(i - 1, j);
                n += 1.0;
            }
            if i + 1 < cs::SECTOR_SIZE.x {
                sum += temp_ctx.get_temp(i + 1, j);
                n += 1.0;
            }
            if j > 0 {
                sum += temp_ctx.get_temp(i, j - 1);
                n += 1.0;
            }
            if j + 1 < cs::SECTOR_SIZE.y {
                sum += temp_ctx.get_temp(i, j + 1);
                n += 1.0;
            }

            extinguish = (sum / n) < BURNING_WOOD_SUSTAIN_TEMP;
        }

        if let Some(temp_ctx) = temp_context.as_deref_mut() {
            temp_ctx.add_temp(i, j + 1, 1.0);
            temp_ctx.add_temp(i, j - 1, 1.0);
            temp_ctx.add_temp(i + 1, j, 1.0);
            temp_ctx.add_temp(i - 1, j, 1.0);
        }
        if prng.next() > 250 {
            try_spawn_smoke(i, j, container, prng, 1);
        }
        if extinguish {
            try_spawn_smoke(i, j, container, prng, 1);
            container[cur] = Wood::id();
            return;
        }
        {
            const SPARK_TRIES_PER_TICK: u8 = 2;
            const SPARK_ATTEMPT_THRESHOLD: u8 = 210;
            const SPARK_IGNITE_THRESHOLD: u8 = 170;

            let mut candidates: [usize; 3] = [0; 3];
            let mut n = 0usize;

            if j + 1 < cs::SECTOR_SIZE.y {
                candidates[n] = cs::xy_to_index(i, j + 1);
                n += 1;
                if i > 0 {
                    candidates[n] = cs::xy_to_index(i - 1, j + 1);
                    n += 1;
                }
                if i + 1 < cs::SECTOR_SIZE.x {
                    candidates[n] = cs::xy_to_index(i + 1, j + 1);
                    n += 1;
                }
            }

            if n != 0 {
                for _ in 0..SPARK_TRIES_PER_TICK {
                    if prng.next() > SPARK_ATTEMPT_THRESHOLD {
                        continue;
                    }
                    let cc = candidates[(prng.next() as usize) % n];
                    if container[cc] == Void::id() && prng.next() > SPARK_IGNITE_THRESHOLD {
                        container[cc] = fire::id();
                    }
                }
            }
        }
        if prng.next() > 25 {
            return;
        }

        if prng.next() > 253 {
            try_spawn_smoke(i, j, container, prng, 1);
            container[cur] = Coal::id();
            return;
        }
    }

    fn stat(&self) -> bool {
        true
    }

    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> &str {
        "burning wood"
    }
    fn id(&self) -> CellType {
        6
    }
}
