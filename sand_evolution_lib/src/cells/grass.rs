use crate::cells::dry_grass::DryGrass;
use crate::cs::{self, PointType};

use super::{burning_wood, sand::Base, void::Void, water::Water, base_water::BaseWater, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Grass;
impl Grass {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        70
    }
}
impl CellTrait for Grass {
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
        // Проверка на засыхание при контакте со щёлочью
        if prng.next() > 180 {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // Проверяем соседние клетки на наличие щёлочи или щелочной воды
            if top_v == Base::id() || top_v == BaseWater::id() ||
               down_v == Base::id() || down_v == BaseWater::id() ||
               r_v == Base::id() || r_v == BaseWater::id() ||
               l_v == Base::id() || l_v == BaseWater::id() {
                // Трава засыхает и превращается в сухую траву
                container[cur] = DryGrass::id();
                return;
            }
        }

        // Медленное смешивание с сухой травой
        if prng.next() > 240 {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // Если рядом есть сухая трава, зелёная трава медленно превращается в сухую
            if top_v == DryGrass::id() || down_v == DryGrass::id() ||
               r_v == DryGrass::id() || l_v == DryGrass::id() {
                // С очень маленькой вероятностью зелёная трава засыхает
                if prng.next() > 250 {
                    container[cur] = DryGrass::id();
                    return;
                }
            }
        }

        // Трава растёт на соседние пустые клетки, если рядом с травой есть вода
        // Проверяем редко для медленного роста
        if prng.next() > 200 {
            // Сначала проверяем, есть ли вода рядом с текущей травой
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let top_v = container[top];
            let down_v = container[down];
            let r_v = container[r];
            let l_v = container[l];

            // Если рядом с травой есть вода, она может расти
            let has_water_nearby = top_v == Water::id() || down_v == Water::id() ||
                                   r_v == Water::id() || l_v == Water::id();

            if has_water_nearby {
                // Проверяем соседние пустые клетки для роста
                let neighbors = [
                    (top, top_v),
                    (down, down_v),
                    (r, r_v),
                    (l, l_v),
                ];

                // Ищем пустую клетку для роста
                for (neighbor_idx, neighbor_v) in neighbors.iter() {
                    if *neighbor_v == Void::id() {
                        // С небольшой вероятностью трава растёт на эту пустую клетку
                        if prng.next() > 220 {
                            container[*neighbor_idx] = Grass::id();
                            return;
                        }
                    }
                }
            }
        }
    }

    fn stat(&self) -> bool {
        true
    }
    fn burnable(&self) -> u8 {
        DryGrass::id()
    }

    fn proton_transfer(&self) -> CellType {
        burning_wood::id()
    }
    fn heatable(&self) -> u8 {
        DryGrass::id()
    }
    fn name(&self) -> &str {
        "grass"
    }
    fn id(&self) -> CellType {
        Grass::id()
    }
}
