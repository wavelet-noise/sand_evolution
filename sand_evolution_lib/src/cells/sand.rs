use crate::cells::base_water::BaseWater;
use crate::cells::salty_water::SaltyWater;
use crate::cs::PointType;

use super::{
    gas::Gas, helper::sand_falling_helper, CellRegistry, CellTrait, CellType, Prng,
    TemperatureContext,
};

pub struct Sand;
impl Sand {
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
impl CellTrait for Sand {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        _: Option<&mut TemperatureContext>,
    ) {
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
    }
    fn den(&self) -> i8 {
        10
    }
    fn proton_transfer(&self) -> CellType {
        Gas::id()
    }
    fn id(&self) -> CellType {
        1
    }
    fn name(&self) -> &str {
        "sand"
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
        _: Option<&mut TemperatureContext>,
    ) {
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
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
    fn name(&self) -> &str {
        "salt"
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
        _: Option<&mut TemperatureContext>,
    ) {
        sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng);
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
    fn name(&self) -> &str {
        "base"
    }
}
