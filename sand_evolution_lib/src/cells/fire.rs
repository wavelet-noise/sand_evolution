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
        mut temp_context: Option<&mut TemperatureContext>,
    ) {
        if prng.next() > 128 {
            // Fire constantly heats the surrounding environment
            if let Some(temp_ctx) = temp_context.as_deref_mut() {
                // Constantly give heat to neighbors
                (temp_ctx.add_temp)(i, j + 1, 1.5); // top
                (temp_ctx.add_temp)(i, j - 1, 1.5); // bottom
                (temp_ctx.add_temp)(i + 1, j, 1.5); // right
                (temp_ctx.add_temp)(i - 1, j, 1.5); // left
            }
            return;
        }

        // Fire is finite - when disappearing gives additional heat to neighboring cells
        if prng.next() > 200 {
            // Give heat to neighboring cells before disappearing
            if let Some(temp_ctx) = temp_context.as_deref_mut() {
                // Give additional heat to neighbors when disappearing
                (temp_ctx.add_temp)(i, j + 1, 2.5); // top
                (temp_ctx.add_temp)(i, j - 1, 2.5); // bottom
                (temp_ctx.add_temp)(i + 1, j, 2.5); // right
                (temp_ctx.add_temp)(i - 1, j, 2.5); // left
            }
            container[cur] = 0;
            return;
        }
        
        // Fire constantly heats the surrounding environment
        if let Some(temp_ctx) = temp_context.as_deref_mut() {
            // Constantly give heat to neighbors
            (temp_ctx.add_temp)(i, j + 1, 1.5); // top
            (temp_ctx.add_temp)(i, j - 1, 1.5); // bottom
            (temp_ctx.add_temp)(i + 1, j, 1.5); // right
            (temp_ctx.add_temp)(i - 1, j, 1.5); // left
        }

        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

        // Pick a neighbor (track both index and coordinates for temperature checks).
        let arr = [(i, j + 1, top), (i, j - 1, down), (i - 1, j, l), (i + 1, j, r)];
        let (nx, ny, cc) = arr[(prng.next() % 4) as usize];

        // Fire does NOT directly ignite/transform neighbors on contact.
        // It only heats (handled above) and moves/disappears.

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
