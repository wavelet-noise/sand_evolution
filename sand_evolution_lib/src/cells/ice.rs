use crate::cs::{PointType, self};

use super::{
    gas::Gas,
    helper::sand_faling_helper,
    water::{BaseWater, SaltyWater, Water},
    CellRegistry, CellTrait, CellType, Prng, void::Void,
};

pub struct Ice;
impl Ice {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        1
    }
}
impl CellTrait for Ice {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        if prng.next() > 1 {
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let bot = cs::xy_to_index(i, j - 1);
        let left = cs::xy_to_index(i + 1, j);
        let right = cs::xy_to_index(i - 1, j);

        let arr = [top, left, right, bot];
        let cc = arr[(prng.next() % 3) as usize];
        let top_v = container[cc];

        if top_v == Void::id() {
            container[cc] = Water::id();
        }
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn den(&self) -> i8 {
        10
    }
    fn id(&self) -> CellType {
        55
    }
    fn name(&self) -> String {
        "ice".to_owned()
    }
}

pub struct Salt;
impl Salt {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        13
    }
}
impl CellTrait for Salt {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }
    fn den(&self) -> i8 {
        10
    }
    fn dissolve(&self) -> CellType {
        SaltyWater::id()
    }
    fn id(&self) -> CellType {
        13
    }
    fn name(&self) -> String {
        "salt".to_owned()
    }
}

pub struct Base;
impl Base {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        14
    }
}
impl CellTrait for Base {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    ) {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }
    fn den(&self) -> i8 {
        10
    }
    fn proton_transfer(&self) -> CellType {
        Salt::id()
    }
    fn dissolve(&self) -> CellType {
        BaseWater::id()
    }
    fn id(&self) -> CellType {
        14
    }
    fn name(&self) -> String {
        "base".to_owned()
    }
}
