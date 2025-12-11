use crate::cs::{self, PointType};

use super::{burning_wood, grass::Grass, water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct DryGrass;
impl DryGrass {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        71
    }
}
impl CellTrait for DryGrass {
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
        // Сухая трава восстанавливается при контакте с водой
        // Проверяем редко для медленного роста
        if prng.next() > 200 {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // Если рядом есть вода, сухая трава превращается в траву
            if top_v == Water::id() || down_v == Water::id() ||
               r_v == Water::id() || l_v == Water::id() {
                container[cur] = Grass::id();
                return;
            }
        }

        // Медленное смешивание с зелёной травой
        if prng.next() > 240 {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // Если рядом есть зелёная трава, сухая трава медленно превращается в зелёную
            if top_v == Grass::id() || down_v == Grass::id() ||
               r_v == Grass::id() || l_v == Grass::id() {
                // С очень маленькой вероятностью сухая трава зеленеет
                if prng.next() > 250 {
                    container[cur] = Grass::id();
                    return;
                }
            }
        }
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> u8 {
        burning_wood::id()
    }

    fn burnable(&self) -> u8 {
        burning_wood::id()
    }
    fn proton_transfer(&self) -> CellType {
        burning_wood::id()
    }
    fn id(&self) -> CellType {
        71
    }
    fn name(&self) -> &str {
        "dry grass"
    }
}
