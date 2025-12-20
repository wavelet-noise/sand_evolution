use crate::cells::helper::fluid_falling_helper;
use crate::cs::{self, PointType};

use super::{void::Void, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Plasma;
impl Plasma {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        61
    }
}
impl CellTrait for Plasma {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

        let arr = [top, down, l, r];
        let cc = arr[(prng.next() % 4) as usize];

        if prng.next() > 50 {
            let cc_v = container[cc] as usize;
            let cc_c = &_pal_container.pal[cc_v];
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

        if prng.next() > 200
            && fluid_falling_helper(self.den(), i, j, container, _pal_container, cur, prng, 10)
        {
            return;
        }
        if prng.next() > 110 {
            let top = cs::xy_to_index(i, j + 1);
            let bot = cs::xy_to_index(i, j - 1);
            let left = cs::xy_to_index(i + 1, j);
            let right = cs::xy_to_index(i - 1, j);

            let arr = [top, left, right, bot];
            let cc = arr[(prng.next() % 4) as usize];
            let rand_v = container[cc];

            if rand_v == Void::id() {
                container[cc] = Plasma::id();
            }
        } else {
            container[cur] = Void::id();
        }
    }

    fn den(&self) -> i8 {
        1
    }
    fn casts_shadow(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "plasma"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
    fn display_color(&self) -> [u8; 3] {
        [153, 77, 255]
    }
}
