use super::{helper::fluid_falling_helper, *};
use crate::cs::PointType;

pub struct Water;
impl Water {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        2
    }
}

impl CellTrait for Water {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim) {
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
        }
    }

    fn den(&self) -> i8 {
        1
    }

    fn name(&self) -> String {
        "water".to_owned()
    }

    fn id(&self) -> CellType {
        2
    }
}


pub struct BaseWater;
impl BaseWater {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        16
    }
}

impl CellTrait for BaseWater {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim) {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let arr = [top, down, l, r];
            let cc = arr[(dim.next() % 4) as usize];

            if dim.next() > 50 {
                let cc_v = container[cc] as usize;
                let cc_c = &pal_container.pal[cc_v];

                // let cc_dis = cc_c.dissolve();
                // if cc_dis != Void::id() {
                //     container[cc] = Void::id();
                //     container[cur] = cc_dis;
                //     return;
                // }

                let cc_pt = cc_c.proton_transfer();
                if cc_pt != Void::id() && cc_v != BaseWater::id() as usize {
                    container[cc] = cc_pt;
                    container[cur] = Water::id();
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        2
    }
    fn proton_transfer(&self) -> CellType {
        SaltyWater::id()
    }
    fn name(&self) -> String {
        "base water".to_owned()
    }

    fn id(&self) -> CellType {
        15
    }
}


pub struct SaltyWater;
impl SaltyWater {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        15
    }
}

impl CellTrait for SaltyWater {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
    ) {
        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim) {
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
        }
    }

    fn den(&self) -> i8 {
        2
    }

    fn name(&self) -> String {
        "salty water".to_owned()
    }

    fn id(&self) -> CellType {
        16
    }
}