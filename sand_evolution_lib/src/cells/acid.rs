use super::{*, helper::fluid_falling_helper};
use crate::cs::PointType;

pub struct Acid;
impl Acid {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        9
    }
}
impl CellTrait for Acid {
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
                let cc_pt = cc_c.proton_transfer();

                if cc_pt != Void::id() {
                    container[cc] = cc_pt;
                    container[cur] = DeluteAcid::id();
                    return;
                }

                if cc_v == Salt::id() as usize {
                    container[cc] = SaltyWater::id();
                    container[cur] = DeluteAcid::id();
                    return;
                }

                let cc_h = cc_c.heatable();

                if cc_h != Void::id() {
                    container[cc] = cc_h;
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        3
    }

    fn name(&self) -> &str {
        "acid"
    }

    fn id(&self) -> CellType {
        9
    }
}
