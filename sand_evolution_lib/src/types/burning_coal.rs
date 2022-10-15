use crate::cs::{self, PointType};
use super::*;

pub const fn new() -> Cell { Cell }
pub fn boxed() -> Box<Cell> { Box::new(new()) }
pub fn id() -> CellType { 7 }

pub struct Cell;
impl CellTrait for Cell {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Dim)
    {
        if !sand_faling_helper(self.den(), i, j, container, pal_container, cur) {

            let bot = cs::xy_to_index(i, j - 1);
            let bot_v = container[bot] as usize;

            let top = cs::xy_to_index(i, j + 1);

            if container[top] == water::id()
            {
                container[top] = steam::id();
                container[cur] = Coal::id();
                return;
            }

            if prng.next() > 200
            {
                return;
            }

            if prng.next() > 250
            {
                container[cur] = Void::id();
                prng.add_carb();
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

            if prng.next() > 50
            {
                return;
            }

            let bot_c = &pal_container.pal[bot_v];
            let bot_b = bot_c.burnable();

            if bot_b != Void::id()
            {
                container[bot] = bot_b;
                return;
            }
        }
    }

    fn den(&self) -> i8 { 2 }
}