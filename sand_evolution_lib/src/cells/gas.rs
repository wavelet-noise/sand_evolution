use super::{
    helper::{fluid_falling_helper, fluid_flying_helper},
    *,
};
use crate::cs::PointType;

pub struct Gas;
impl Gas {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        10
    }
}
impl CellTrait for Gas {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        -1
    }
    fn burnable(&self) -> CellType {
        BurningGas::id()
    }
    fn heatable(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> String {
        "gas".to_owned()
    }

    fn id(&self) -> CellType {
        10
    }
}

pub struct BurningGas;
impl BurningGas {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        11
    }
}
impl CellTrait for BurningGas {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        let topl = cs::xy_to_index(i - 1, j + 1);
        let topr = cs::xy_to_index(i + 1, j + 1);

        let top = cs::xy_to_index(i, j + 1);
        let arr = [top, topl, topr];
        let cc = arr[(dim.next() % 3) as usize];
        let top_v = container[cc];

        let cc_v = container[cc] as usize;
        let cc_c = &pal_container.pal[cc_v];

        let cc_h = cc_c.heatable();

        if cc_h != Void::id() {
            container[cc] = cc_h;
            container[cur] = fire::id();
            return;
        }

        let cc_b = cc_c.burnable();

        if cc_b != Void::id() {
            container[cc] = cc_b;
            container[cur] = fire::id();
            return;
        }

        if top_v == Void::id() {
            container[cc] = fire::id();
        }

        if dim.next() > 240 {
            container[cur] = fire::id();
        }

        fluid_flying_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 {
        -10
    }
    fn name(&self) -> String {
        "burning gas".to_owned()
    }

    fn id(&self) -> CellType {
        11
    }
}
