use crate::cs::{self, PointType};

use super::{void::Void, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Laser;
impl Laser {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        62
    }
}
impl CellTrait for Laser {
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

        if prng.next() < 200 {
            return;
        } else {
            container[cur] = Void::id();
        }
    }

    fn den(&self) -> i8 {
        0
    }
    fn casts_shadow(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "laser"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
    fn display_color(&self) -> [u8; 3] {
        [255, 26, 0]
    }
}
