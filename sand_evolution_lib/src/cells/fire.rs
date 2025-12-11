use super::{*, TemperatureContext};
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
        pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        if prng.next() > 128 {
            // Огонь постоянно нагревает окружающую среду
            if let Some(temp_ctx) = temp_context {
                // Постоянно отдаем тепло соседям
                (temp_ctx.add_temp)(i, j + 1, 3.0); // верх
                (temp_ctx.add_temp)(i, j - 1, 3.0); // низ
                (temp_ctx.add_temp)(i + 1, j, 3.0); // право
                (temp_ctx.add_temp)(i - 1, j, 3.0); // лево
            }
            return;
        }

        // Огонь конечен - при исчезновении отдает дополнительное тепло соседним клеткам
        if prng.next() > 200 {
            // Отдаем тепло соседним клеткам перед исчезновением
            if let Some(temp_ctx) = temp_context {
                // Отдаем дополнительное тепло соседям при исчезновении
                (temp_ctx.add_temp)(i, j + 1, 5.0); // верх
                (temp_ctx.add_temp)(i, j - 1, 5.0); // низ
                (temp_ctx.add_temp)(i + 1, j, 5.0); // право
                (temp_ctx.add_temp)(i - 1, j, 5.0); // лево
            }
            container[cur] = 0;
            return;
        }
        
        // Огонь постоянно нагревает окружающую среду
        if let Some(temp_ctx) = temp_context {
            // Постоянно отдаем тепло соседям
            (temp_ctx.add_temp)(i, j + 1, 3.0); // верх
            (temp_ctx.add_temp)(i, j - 1, 3.0); // низ
            (temp_ctx.add_temp)(i + 1, j, 3.0); // право
            (temp_ctx.add_temp)(i - 1, j, 3.0); // лево
        }

        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

        let arr = [top, down, l, r];
        let cc = arr[(prng.next() % 4) as usize];

        if prng.next() > 50 {
            let cc_v = container[cc] as usize;
            let cc_c = &pal_container.pal[cc_v];
            let cc_b = cc_c.burnable();

            if cc_b != Void::id() {
                container[cc] = cc_b;
                return;
            }

            let cc_h = cc_c.heatable();

            if cc_h != Void::id() && prng.next() > cc_c.heat_proof() {
                container[cc] = cc_h;
                return;
            }
        }

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

        // Огонь исчезает и отдает тепло соседним клеткам
        // temp_context уже использован выше, но если мы дошли сюда, значит он был None или уже использован
        container[cur] = 0;
    }

    fn den(&self) -> i8 {
        -1
    }

    fn name(&self) -> &str {
        "fire"
    }
    fn id(&self) -> CellType {
        4
    }
}
