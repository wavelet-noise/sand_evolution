pub mod acid;
pub mod burning_coal;
pub mod burning_wood;
pub mod coal;
pub mod fire;
pub mod gas;
mod helper;
pub mod sand;
pub mod steam;
pub mod stone;
pub mod void;
pub mod water;
pub mod wood;
pub mod ice;
pub mod crushed_ice;
pub mod snow;

use std::collections::HashMap;

use crate::cs::{self, PointType};

use self::{
    acid::{Acid, DeluteAcid},
    coal::Coal,
    gas::{BurningGas, Gas},
    helper::sand_faling_helper,
    sand::{Base, Salt, Sand},
    void::Void,
    water::{BaseWater, SaltyWater, Water},
    wood::Wood, steam::Steam, burning_coal::BurningCoal, ice::Ice, crushed_ice::CrushedIce, snow::Snow
};
pub type CellType = u8;

pub struct Prng {
    rnd: [u8; 256],
    rnd_next: usize,

    carb: i32,
}

impl Prng {
    pub fn new() -> Self {
        let mut buf = [0u8; 256];
        _ = getrandom::getrandom(&mut buf);
        Self {
            rnd: buf,
            rnd_next: 0,
            carb: 100,
        }
    }

    pub fn gen(&mut self) {
        let mut buf = [0u8; 256];
        _ = getrandom::getrandom(&mut buf);
        self.rnd = buf;
        self.rnd_next = 0;
    }

    pub fn next(&mut self) -> u8 {
        self.rnd_next += 1;
        self.rnd_next = if self.rnd_next >= 256 {
            0
        } else {
            self.rnd_next
        };

        return self.rnd[self.rnd_next];
    }

    pub fn add_carb(&mut self) {
        self.carb += 1;
    }

    pub fn rm_carb(&mut self) {
        self.carb -= 1;
    }

    pub fn carb(&self) -> i32 {
        self.carb
    }
}

pub struct CellRegistry {
    pub pal: Vec<Box<dyn CellTrait>>,

    pub dict: HashMap<String, u8>,
}

impl CellRegistry {
    pub fn new() -> Self {
        let mut me = Self {
            pal: Vec::new(),
            dict: HashMap::new(),
        };
        setup_palette(&mut me);
        return me;
    }
}

pub trait CellTrait {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
    );
    fn den(&self) -> i8 {
        0
    }
    fn stat(&self) -> bool {
        false
    }
    fn burnable(&self) -> CellType {
        Void::id()
    }
    fn proton_transfer(&self) -> CellType {
        Void.id()
    }
    fn dissolve(&self) -> CellType {
        Void.id()
    }
    fn heatable(&self) -> CellType {
        Void::id()
    }
    fn name(&self) -> String {
        "".to_owned()
    }
    fn id(&self) -> CellType {
        0
    }
}

pub fn setup_palette(cell_registry: &mut CellRegistry) {
    for _ in 0..=255 {
        cell_registry.pal.push(Void::boxed())
    }

    cell_registry.pal[1] = Sand::boxed();
    cell_registry.pal[2] = Water::boxed();
    cell_registry.pal[3] = Steam::boxed();
    cell_registry.pal[4] = fire::boxed();
    cell_registry.pal[6] = burning_wood::boxed();
    cell_registry.pal[7] = BurningCoal::boxed();
    cell_registry.pal[8] = Coal::boxed();
    cell_registry.pal[9] = Acid::boxed();
    cell_registry.pal[10] = Gas::boxed();
    cell_registry.pal[11] = BurningGas::boxed();
    cell_registry.pal[12] = DeluteAcid::boxed();
    cell_registry.pal[13] = Salt::boxed();
    cell_registry.pal[14] = Base::boxed();
    cell_registry.pal[15] = SaltyWater::boxed();
    cell_registry.pal[16] = BaseWater::boxed();
    cell_registry.pal[50] = Wood::boxed();
    cell_registry.pal[55] = Ice::boxed();
    cell_registry.pal[56] = CrushedIce::boxed();
    cell_registry.pal[57] = Snow::boxed();
    cell_registry.pal[255] = stone::Stone::boxed();

    let mut index = 0;
    for a in cell_registry.pal.iter() {
        cell_registry.dict.insert(a.name(), index);
        index = index.wrapping_add(1)
    }
}
