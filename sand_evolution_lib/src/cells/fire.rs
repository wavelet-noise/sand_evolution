use super::{helper::try_spawn_smoke, TemperatureContext, *};
use crate::cs::{self, PointType};

pub const fn new() -> Cell {
    Cell
}
pub fn boxed() -> Box<Cell> {
    Box::new(new())
}
pub fn id() -> CellType {
    4
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
        const FIRE_SUSTAIN_TEMP: f32 = 80.0;
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

            extinguish = (sum / n) < FIRE_SUSTAIN_TEMP;
        }

        if prng.next() > 128 {
            if let Some(temp_ctx) = temp_context.as_deref_mut() {
                (temp_ctx.add_temp)(i, j + 1, 1.5);
                (temp_ctx.add_temp)(i, j - 1, 1.5);
                (temp_ctx.add_temp)(i + 1, j, 1.5);
                (temp_ctx.add_temp)(i - 1, j, 1.5);
            }
            if prng.next() > 240 {
                try_spawn_smoke(i, j, container, prng, 1);
            }
            if extinguish {
                try_spawn_smoke(i, j, container, prng, 1);
                container[cur] = Void::id();
            }
            return;
        }

        if prng.next() > 200 {
            if let Some(temp_ctx) = temp_context.as_deref_mut() {
                (temp_ctx.add_temp)(i, j + 1, 2.5);
                (temp_ctx.add_temp)(i, j - 1, 2.5);
                (temp_ctx.add_temp)(i + 1, j, 2.5);
                (temp_ctx.add_temp)(i - 1, j, 2.5);
            }
            try_spawn_smoke(i, j, container, prng, 1);
            container[cur] = 0;
            return;
        }

        if let Some(temp_ctx) = temp_context.as_deref_mut() {
            (temp_ctx.add_temp)(i, j + 1, 1.5);
            (temp_ctx.add_temp)(i, j - 1, 1.5);
            (temp_ctx.add_temp)(i + 1, j, 1.5);
            (temp_ctx.add_temp)(i - 1, j, 1.5);
        }
        if prng.next() > 240 {
            try_spawn_smoke(i, j, container, prng, 1);
        }
        if extinguish {
            try_spawn_smoke(i, j, container, prng, 1);
            container[cur] = Void::id();
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

        let arr = [
            (i, j + 1, top),
            (i, j - 1, down),
            (i - 1, j, l),
            (i + 1, j, r),
        ];
        let (_nx, _ny, _cc) = arr[(prng.next() % 4) as usize];

        let top_v = container[top];

        if top_v == Void::id() {
            container.swap(cur, top);
            return;
        }

        let topl = cs::xy_to_index(i - 1, j + 1);
        let topl_v = container[topl];

        if topl_v == Void::id() {
            container.swap(cur, topl);
            return;
        }

        let topr = cs::xy_to_index(i + 1, j + 1);
        let topr_v = container[topr];

        if topr_v == Void::id() {
            container.swap(cur, topr);
            return;
        }

        try_spawn_smoke(i, j, container, prng, 1);
        container[cur] = 0;
    }

    fn den(&self) -> i8 {
        -1
    }
    fn casts_shadow(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "fire"
    }
    fn id(&self) -> CellType {
        4
    }
    fn display_color(&self) -> [u8; 3] {
        [255, 128, 0]
    }
}
