use crate::cs::{self, PointType};

use super::{
    void::Void, water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
};

pub struct CrushedIce;
impl CrushedIce {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        56
    }
}
impl CellTrait for CrushedIce {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        // Дробленый лед тает только на основе температуры - если температура > 0, то тает и охлаждает среду
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Если температура выше 0 градусов, дробленый лед тает
            // Проверяем редко, чтобы избежать быстрых циклов таяния/замерзания
            if prng.next() > 200 {
                if temperature > 0.0 {
                    // Отдаем холод соседям при таянии (уменьшено, чтобы избежать вспышек)
                    (temp_ctx.add_temp)(i, j + 1, -3.0); // верх
                    (temp_ctx.add_temp)(i, j - 1, -3.0); // низ
                    (temp_ctx.add_temp)(i + 1, j, -3.0); // лево
                    (temp_ctx.add_temp)(i - 1, j, -3.0); // право
                    
                    container[cur] = Water::id();
                    return;
                }
            }
        }
        
        // Специальная логика для крушеного льда: плавает на воде (плотность 0), но падает через Void
        // Проверяем клетку внизу
        let down = cs::xy_to_index(i, j - 1);
        let down_v = container[down] as usize;
        let down_c = &pal_container.pal[down_v];
        
        // Если внизу Void, падаем вниз
        if down_v == Void::id() as usize {
            container.swap(cur, down);
            return;
        }
        
        // Если внизу что-то с плотностью меньше 0 (легче крушеного льда), падаем
        if down_c.den() < self.den() && !down_c.stat() {
            container.swap(cur, down);
            return;
        }
        
        // Проверяем диагонали вниз
        const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
        let selected_order = ORDER[(prng.next() % 2) as usize];
        
        for k in 0..2 {
            match selected_order[k] {
                0 => {
                    let dr = cs::xy_to_index(i + 1, j - 1);
                    let dr_v = container[dr] as usize;
                    let dr_c = &pal_container.pal[dr_v];
                    if dr_v == Void::id() as usize || (dr_c.den() < self.den() && !dr_c.stat()) {
                        container.swap(cur, dr);
                        return;
                    }
                }
                1 => {
                    let dl = cs::xy_to_index(i - 1, j - 1);
                    let dl_v = container[dl] as usize;
                    let dl_c = &pal_container.pal[dl_v];
                    if dl_v == Void::id() as usize || (dl_c.den() < self.den() && !dl_c.stat()) {
                        container.swap(cur, dl);
                        return;
                    }
                }
                _ => (),
            }
        }

        if prng.next() > 1 {
            return;
        }

        if prng.next() > 32 {
            return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let bot = cs::xy_to_index(i, j - 1);
        let left = cs::xy_to_index(i + 1, j);
        let right = cs::xy_to_index(i - 1, j);

        // Crushed ice should not turn into water on contact
        // Only melting at temperature > 0
    }

    fn den(&self) -> i8 {
        0
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn heat_proof(&self) -> u8 {
        200
    }
    fn name(&self) -> &str {
        "crushed ice"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
