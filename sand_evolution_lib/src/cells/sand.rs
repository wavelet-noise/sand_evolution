use crate::cs::PointType;

use super::{gas::Gas, helper::sand_faling_helper, CellRegistry, CellTrait, CellType, Prng};

pub struct Sand;
impl Sand {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
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
    ) {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur, prng);
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
    fn name(&self) -> String {
        "sand".to_owned()
    }
}

pub struct Snow;
impl Snow {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
}
impl CellTrait for Snow {
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
        2
    }
    fn id(&self) -> CellType {
        1
    }
    fn name(&self) -> String {
        "snow".to_owned()
    }
}
