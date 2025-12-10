pub mod acid;
pub mod burning_coal;
pub mod burning_gas;
pub mod burning_wood;
pub mod coal;
pub mod crushed_ice;
pub mod electricity;
pub mod fire;
pub mod gas;
mod helper;
pub mod ice;
pub mod sand;
pub mod snow;
pub mod steam;
pub mod stone;
pub mod void;
pub mod water;
pub mod wood;
mod base_water;
mod salty_water;
mod delute_acid;
mod grass;

mod laser;

mod dry_grass;
mod plasma;

use crate::cells::electricity::Electricity;
use std::collections::HashMap;
use base_water::BaseWater;
use delute_acid::DeluteAcid;
use salty_water::SaltyWater;
use crate::cells::dry_grass::DryGrass;
use crate::cells::grass::Grass;
use crate::cells::laser::Laser;
use crate::cells::plasma::Plasma;
use crate::cs::{self, PointType};

use self::{
    acid::Acid,
    burning_coal::BurningCoal,
    burning_gas::BurningGas,
    coal::Coal,
    crushed_ice::CrushedIce,
    gas::Gas,
    helper::sand_falling_helper,
    ice::Ice,
    sand::{Base, Salt, Sand},
    snow::Snow,
    steam::Steam,
    void::Void,
    water::Water,
    wood::Wood,
};
pub type CellType = u8;

pub struct Prng {
    rnd: [u8; 256],
    rnd_next: usize,
    carb: i32,
    state: [u64; 4],
}

impl Prng {
    pub fn new() -> Self {
        // Инициализация хорошего PRNG (xoshiro256**) из системного источника энтропии
        // через getrandom 0.2 (один раз на создание).
        let mut seed_bytes = [0u8; 32];
        let _ = getrandom::getrandom(&mut seed_bytes);

        let mut state = [0u64; 4];
        for i in 0..4 {
            let mut chunk = [0u8; 8];
            chunk.copy_from_slice(&seed_bytes[i * 8..(i + 1) * 8]);
            state[i] = u64::from_le_bytes(chunk);
        }
        // Запретить нулевое состояние (xoshiro требует ненулевой стейт).
        if state.iter().all(|&x| x == 0) {
            state = [
                0x0123_4567_89AB_CDEF,
                0x89AB_CDEF_0123_4567,
                0xF00D_F00D_DEAD_BEEF,
                0xC0DE_CAFE_1337_0001,
            ];
        }

        let mut prng = Self {
            rnd: [0u8; 256],
            rnd_next: 0,
            carb: 100,
            state,
        };
        prng.gen();
        prng
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // xoshiro256** (см. http://prng.di.unimi.it/)
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }

    pub fn gen(&mut self) {
        let mut idx = 0;
        let total_len = self.rnd.len();
        while idx < total_len {
            let x = self.next_u64().to_le_bytes();
            let remaining = total_len - idx;
            let copy_len = if remaining < 8 { remaining } else { 8 };
            self.rnd[idx..idx + copy_len].copy_from_slice(&x[..copy_len]);
            idx += copy_len;
        }
        self.rnd_next = 0;
    }

    pub fn next(&mut self) -> u8 {
        self.rnd_next += 1;
        if self.rnd_next >= self.rnd.len() {
            // Если вышли за буфер — генерируем новый блок.
            self.gen();
        }
        self.rnd[self.rnd_next]
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
        me
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
    fn heat_proof(&self) -> u8 {
        1
    }
    fn name(&self) -> &str {
        ""
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
    cell_registry.pal[60] = Electricity::boxed();
    cell_registry.pal[61] = Plasma::boxed();
    cell_registry.pal[62] = Laser::boxed();
    cell_registry.pal[70] = Grass::boxed();
    cell_registry.pal[71] = DryGrass::boxed();
    cell_registry.pal[255] = stone::Stone::boxed();

    let mut index = 0;
    for a in cell_registry.pal.iter() {
        cell_registry.dict.insert(a.name().to_owned(), index);
        index = index.wrapping_add(1)
    }
}
