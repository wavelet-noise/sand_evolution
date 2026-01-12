pub mod acid;
mod base_water;
pub mod black_hole;
pub mod burning_coal;
pub mod burning_gas;
pub mod burning_powder;
pub mod burning_wood;
pub mod coal;
pub mod crushed_ice;
pub mod powder;
mod delute_acid;
pub mod earth;
pub mod electricity;
pub mod fire;
pub mod gas;
pub mod copper;
mod grass;
pub mod gravel;
mod helper;
pub mod ice;
pub mod liquid_gas;
mod salty_water;
pub mod salt;
pub mod sand;
pub mod smoke;
pub mod snow;
pub mod steam;
pub mod stone;
pub mod void;
pub mod water;
pub mod wood;

mod laser;

mod dry_grass;
mod plasma;

use crate::cells::dry_grass::DryGrass;
use crate::cells::electricity::Electricity;
use crate::cells::grass::Grass;
use crate::cells::laser::Laser;
use crate::cells::plasma::Plasma;
use crate::cs::{self, PointType};
use base_water::BaseWater;
use delute_acid::DeluteAcid;
use salty_water::SaltyWater;
use std::collections::HashMap;

use self::{
    acid::Acid,
    black_hole::BlackHole,
    burning_coal::BurningCoal,
    burning_gas::BurningGas,
    burning_powder::BurningPowder,
    copper::Copper,
    coal::Coal,
    crushed_ice::CrushedIce,
    powder::Powder,
    earth::Earth,
    gas::Gas,
    gravel::Gravel,
    helper::sand_falling_helper,
    ice::Ice,
    liquid_gas::LiquidGas,
    salt::Salt,
    sand::{Base, Sand},
    smoke::Smoke,
    snow::Snow,
    steam::Steam,
    void::Void,
    water::Water,
    wood::Wood,
};
pub type CellType = u8;

pub const PRNG_POOL_SIZE: usize = 2048;

pub struct Prng {
    rnd: [u8; PRNG_POOL_SIZE],
    rnd_next: usize,
    carb: i32,
}

impl Prng {
    pub fn new() -> Self {
        let mut buf = [0u8; PRNG_POOL_SIZE];
        let _ = getrandom::getrandom(&mut buf);
        Self {
            rnd: buf,
            rnd_next: 0,
            carb: 100,
        }
    }

    pub fn gen(&mut self) {
        let _ = getrandom::getrandom(&mut self.rnd);
        self.rnd_next = 0;
    }

    pub fn next(&mut self) -> u8 {
        self.rnd_next += 1;
        self.rnd_next = if self.rnd_next >= PRNG_POOL_SIZE {
            0
        } else {
            self.rnd_next
        };
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

pub struct TemperatureContext<'a> {
    pub get_temp: Box<dyn Fn(PointType, PointType) -> f32 + 'a>,
    pub add_temp: Box<dyn FnMut(PointType, PointType, f32) + 'a>,
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
        temp_context: Option<&mut TemperatureContext>,
    );
    fn den(&self) -> i8 {
        0
    }
    fn stat(&self) -> bool {
        false
    }
    /// Whether this cell type casts directional shadows in the post-process shader.
    /// Default: true (override to disable for "flying"/non-occluding particles like gas, fire, etc.).
    fn casts_shadow(&self) -> bool {
        true
    }
    /// Per-cell-type shadow "color" and opacity used by the post-process shader.
    ///
    /// Encoding: RGBA8 where:
    /// - RGB is a multiplier applied to the background color in shadow (255 = no darkening),
    /// - A is the shadow opacity/transmittance strength (0 = does not affect shadows, 255 = fully affects).
    ///
    /// Defaults:
    /// - if `casts_shadow()==false` => no contribution to shadows,
    /// - otherwise => neutral dark shadow similar to the previous hardcoded behavior.
    fn shadow_rgba(&self) -> [u8; 4] {
        if !self.casts_shadow() {
            return [255, 255, 255, 0];
        }
        [140, 140, 140, 255]
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
    fn ignition_temperature(&self) -> Option<f32> {
        None
    }
    fn thermal_conductivity(&self) -> f32 {
        0.0
    }
    fn convection_factor(&self) -> f32 {
        0.0
    }
    fn display_color(&self) -> [u8; 3] {
        [200, 200, 200]
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
    cell_registry.pal[5] = Wood::boxed();
    cell_registry.pal[6] = burning_wood::boxed();
    cell_registry.pal[7] = BurningCoal::boxed();
    cell_registry.pal[8] = Coal::boxed();
    cell_registry.pal[9] = Acid::boxed();
    cell_registry.pal[10] = Gas::boxed();
    cell_registry.pal[11] = BurningGas::boxed();
    cell_registry.pal[13] = Salt::boxed();
    cell_registry.pal[14] = Base::boxed();
    cell_registry.pal[17] = LiquidGas::boxed();
    cell_registry.pal[18] = Earth::boxed();
    cell_registry.pal[19] = Gravel::boxed();
    cell_registry.pal[20] = Copper::boxed();
    cell_registry.pal[21] = Smoke::boxed();
    cell_registry.pal[50] = Powder::boxed();
    cell_registry.pal[51] = BurningPowder::boxed();
    cell_registry.pal[55] = Ice::boxed();
    cell_registry.pal[56] = CrushedIce::boxed();
    cell_registry.pal[57] = Snow::boxed();
    cell_registry.pal[60] = Electricity::boxed();
    cell_registry.pal[61] = Plasma::boxed();
    cell_registry.pal[62] = Laser::boxed();
    cell_registry.pal[70] = Grass::boxed();
    cell_registry.pal[71] = DryGrass::boxed();
    cell_registry.pal[80] = BlackHole::boxed();
    cell_registry.pal[83] = DeluteAcid::boxed();
    cell_registry.pal[84] = SaltyWater::boxed();
    cell_registry.pal[85] = BaseWater::boxed();
    cell_registry.pal[255] = stone::Stone::boxed();

    let mut index = 0;
    for a in cell_registry.pal.iter() {
        cell_registry.dict.insert(a.name().to_owned(), index);
        index = index.wrapping_add(1)
    }
}
