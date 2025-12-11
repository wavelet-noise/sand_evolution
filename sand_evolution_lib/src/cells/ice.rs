use crate::cells::crushed_ice::CrushedIce;
use crate::cs::{self, PointType};

use super::{void::Void, water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext};

pub struct Ice;
impl Ice {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        55
    }
}
impl CellTrait for Ice {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        _pal_container: &CellRegistry,
        prng: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        // Лед тает только на основе температуры - если температура > 0, то тает и охлаждает среду
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Если температура выше 0 градусов, лед тает
            if temperature > 0.0 {
                // Лед тает и отдает холод (поглощает тепло) соседним клеткам
                // Уменьшено, чтобы избежать цепной реакции
                (temp_ctx.add_temp)(i, j + 1, -2.0); // верх
                (temp_ctx.add_temp)(i, j - 1, -2.0); // низ
                (temp_ctx.add_temp)(i + 1, j, -2.0); // лево
                (temp_ctx.add_temp)(i - 1, j, -2.0); // право
                
                container[cur] = Water::id();
                return;
            }
            
            // Лед активно замораживает соседнюю воду через контакт
            // Проверяем соседние клетки на наличие воды и понижаем их температуру
            // Делаем это достаточно часто, но не каждый кадр для баланса
            if prng.next() > 200 {
                let top_idx = cs::xy_to_index(i, j + 1);
                let bot_idx = cs::xy_to_index(i, j - 1);
                let left_idx = cs::xy_to_index(i + 1, j);
                let right_idx = cs::xy_to_index(i - 1, j);
                
                // Значительно понижаем температуру соседней воды для быстрого замерзания
                if container[top_idx] == Water::id() {
                    (temp_ctx.add_temp)(i, j + 1, -8.0); // верх
                }
                if container[bot_idx] == Water::id() {
                    (temp_ctx.add_temp)(i, j - 1, -8.0); // низ
                }
                if container[left_idx] == Water::id() {
                    (temp_ctx.add_temp)(i + 1, j, -8.0); // лево
                }
                if container[right_idx] == Water::id() {
                    (temp_ctx.add_temp)(i - 1, j, -8.0); // право
                }
            }
        } else {
            // Если нет системы температуры, используем старую логику прямого замерзания
            let top = cs::xy_to_index(i, j + 1);
            let bot = cs::xy_to_index(i, j - 1);
            let left = cs::xy_to_index(i + 1, j);
            let right = cs::xy_to_index(i - 1, j);
            
            let top_v = container[top];
            let bot_v = container[bot];
            let left_v = container[left];
            let right_v = container[right];
            
            // Проверяем соседние клетки на наличие воды и замораживаем напрямую
            if prng.next() > 240 {
                if top_v == Water::id() {
                    container[top] = Ice::id();
                    return;
                }
                if bot_v == Water::id() {
                    container[bot] = Ice::id();
                    return;
                }
                if left_v == Water::id() {
                    container[left] = Ice::id();
                    return;
                }
                if right_v == Water::id() {
                    container[right] = Ice::id();
                    return;
                }
            }
        }

        // Лед не должен превращаться в воду при контакте с пустотой или водой
        // Только таяние при температуре > 0
    }

    fn den(&self) -> i8 {
        10
    }

    fn stat(&self) -> bool {
        true
    }

    fn heatable(&self) -> CellType {
        Water::id()
    }

    fn heat_proof(&self) -> u8 {
        240
    }
    fn name(&self) -> &str {
        "ice"
    }
    fn id(&self) -> CellType {
        Self::id()
    }
}
