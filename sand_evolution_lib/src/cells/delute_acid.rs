use crate::cells::{CellRegistry, CellTrait, CellType, Prng};
use crate::cells::helper::fluid_falling_helper;
use crate::cells::void::Void;
use crate::cells::water::Water;
use crate::cs;
use crate::cs::PointType;

pub struct DeluteAcid;

impl DeluteAcid {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        12
    }
}

impl CellTrait for DeluteAcid {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1) {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let arr = [top, down, l, r];
            let cc = arr[(dim.next() % 4) as usize];

            if dim.next() > 50 {
                let cc_v = container[cc] as usize;
                let cc_c = &pal_container.pal[cc_v];
                let cc_pt = cc_c.dissolve();

                if cc_pt != Void::id() {
                    container[cc] = Void::id();
                    container[cur] = cc_pt;
                    return;
                }
            }

            if dim.next() > 240 {
                let cc_v = container[cc] as usize;
                let cc_c = &pal_container.pal[cc_v];
                let cc_pt = cc_c.proton_transfer();

                if cc_pt != Void::id() {
                    container[cc] = cc_pt;

                    if dim.next() > 120 {
                        container[cur] = Water::id();
                    } else {
                        container[cur] = Void::id();
                    }
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        2
    }

    fn name(&self) -> &str {
        "delute acid"
    }

    fn id(&self) -> CellType {
        12
    }
}
