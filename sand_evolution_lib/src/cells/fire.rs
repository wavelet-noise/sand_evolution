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
            // Fire constantly heats the surrounding environment
            if let Some(temp_ctx) = temp_context {
                // Constantly give heat to neighbors
                (temp_ctx.add_temp)(i, j + 1, 3.0); // top
                (temp_ctx.add_temp)(i, j - 1, 3.0); // bottom
                (temp_ctx.add_temp)(i + 1, j, 3.0); // right
                (temp_ctx.add_temp)(i - 1, j, 3.0); // left
            }
            return;
        }

        // Fire is finite - when disappearing gives additional heat to neighboring cells
        if prng.next() > 200 {
            // Give heat to neighboring cells before disappearing
            if let Some(temp_ctx) = temp_context {
                // Give additional heat to neighbors when disappearing
                (temp_ctx.add_temp)(i, j + 1, 5.0); // top
                (temp_ctx.add_temp)(i, j - 1, 5.0); // bottom
                (temp_ctx.add_temp)(i + 1, j, 5.0); // right
                (temp_ctx.add_temp)(i - 1, j, 5.0); // left
            }
            container[cur] = 0;
            return;
        }
        
        // Fire constantly heats the surrounding environment
        if let Some(temp_ctx) = temp_context {
            // Constantly give heat to neighbors
            (temp_ctx.add_temp)(i, j + 1, 3.0); // top
            (temp_ctx.add_temp)(i, j - 1, 3.0); // bottom
            (temp_ctx.add_temp)(i + 1, j, 3.0); // right
            (temp_ctx.add_temp)(i - 1, j, 3.0); // left
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

        // Fire disappears and gives heat to neighboring cells
        // temp_context already used above, but if we reached here, it was None or already used
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
