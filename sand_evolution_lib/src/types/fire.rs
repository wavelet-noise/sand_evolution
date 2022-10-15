use crate::cs::{self, PointType};
use super::*;

pub const fn new() -> Cell { Cell }
pub fn boxed() -> Box<Cell> { Box::new(new()) }
pub fn id() -> CellType { 4 }

pub struct Cell;
impl CellTrait for Cell {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Dim)
    {
        if prng.next() > 128
        {
		    return;
        }

        if prng.next() > 200
        {
            container[cur] = 0;
		    return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

	    let arr = [top, down, l, r];
	    let cc = arr[(prng.next() % 4) as usize];

        if prng.next() > 50
        {
            let cc_v = container[cc] as usize;
            let cc_c = &pal_container.pal[cc_v];
            let cc_b = cc_c.burnable();

            if cc_b != Void::id()
            {
                container[cc] = cc_b;
                return;
            }

            let cc_h = cc_c.heatable();

            if cc_h != Void::id()
            {
                container[cc] = cc_h;
                return;
            }
        }

        let top_v = container[top];

        if top_v == Void::id()
        {
            container.swap(cur, top);
            return;
        }

        let topl = cs::xy_to_index(i - 1, j + 1);
        let topl_v = container[topl];

        if topl_v == Void::id()
        {
            container.swap(cur, topl);
            return;
        }

        let topr = cs::xy_to_index(i + 1, j + 1);
        let topr_v = container[topr];

        if topr_v == Void::id()
        {
            container.swap(cur, topr);
            return;
        }

        container[cur] = 0;
    }

    fn den(&self) -> i8 { -1 }
}