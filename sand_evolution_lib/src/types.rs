mod helper;
pub mod burning_wood;
pub mod water;
pub mod steam;
pub mod fire;
pub mod burning_coal;

use crate::cs::{self, PointType};

use self::helper::sand_faling_helper;
pub type CellType = u8;

pub struct Dim {
    rnd: [u8; 256],
    rnd_next: usize,

    carb: i32
}

impl Dim {
    pub fn new() -> Self { 
        let mut buf = [0u8; 256];
        _ = getrandom::getrandom(&mut buf);
        Self { rnd: buf, rnd_next: 0, carb: 100 }
    }

    pub fn gen(&mut self) {
        let mut buf = [0u8; 256];
        _ = getrandom::getrandom(&mut buf);
        self.rnd = buf;
        self.rnd_next = 0;
    }

    pub fn next(&mut self) -> u8 { 
        self.rnd_next += 1; 
        self.rnd_next = if self.rnd_next >= 256 { 0 } else {  self.rnd_next };

        return self.rnd[self.rnd_next];
    }

    pub fn add_carb(&mut self) {
        self.carb += 1; 
    }

    pub fn rm_carb(&mut self) {
        self.carb -= 1; 
    }

    pub fn carb(&self) -> i32 { self.carb }
}

pub struct Palette {
    pub pal: Vec<Box<dyn CellTrait>>,
}

impl Palette {
    pub fn new() -> Self { 
        let mut me = Self { pal: Vec::new() }; 
        setup_palette(&mut me); 
        return me; 
    }
}

pub trait CellTrait {
    fn update(&self, i: PointType, j: PointType, cur : usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Dim);
    fn den(&self) -> i8 { 0 }
    fn stat(&self) -> bool { false }
    fn burnable(&self) -> CellType { Void::id() }
    fn heatable(&self) -> CellType { Void::id() }
}

pub fn setup_palette(pal_container: &mut Palette)
{
    for i in 0..=255
    {
        pal_container.pal.push(Void::boxed())
    }

    pal_container.pal[1] = Sand::boxed();
    pal_container.pal[2] = water::boxed();
    pal_container.pal[3] = steam::boxed();
    pal_container.pal[4] = fire::boxed();
    pal_container.pal[5] = Wood::boxed();
    pal_container.pal[6] = burning_wood::boxed();
    pal_container.pal[7] = burning_coal::boxed();
    pal_container.pal[8] = Coal::boxed();
    pal_container.pal[255] = Stone::boxed();
}

pub struct Void;
impl Void {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 0 }
}
impl CellTrait for Void {
    fn update(&self, _: PointType, _: PointType, _: usize, _: & mut [u8], _: &Palette, _: &mut Dim)
    {
        
    }
}

pub struct Sand;
impl Sand {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 1 }
}
impl CellTrait for Sand {
     fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, _: &mut Dim)
     {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur);
     }

     fn den(&self) -> i8 { 2 }
}

pub struct Stone;
impl Stone {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 255 }
}
impl CellTrait for Stone {
    fn update(&self, _: PointType, _: PointType, _ : usize, _: & mut [CellType], _: &Palette, _: &mut Dim)
    {
    
    }

    fn stat(&self) -> bool { true }
}

pub struct Wood;
impl Wood {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 5 }
}
impl CellTrait for Wood {

    fn update(&self, _: PointType, _: PointType, _: usize, _: & mut [CellType], _: &Palette, _: &mut Dim)
    {
        
    }

    fn stat(&self) -> bool { true }
    fn burnable(&self) -> u8 { burning_wood::id() }
}

pub struct Coal;
impl Coal {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 8 }
}
impl CellTrait for Coal {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Dim)
    {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur);
    }

    fn den(&self) -> i8 { 2 }
    fn burnable(&self) -> u8 { burning_coal::id() }
}