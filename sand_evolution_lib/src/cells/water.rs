use super::{*, helper::fluid_falling_helper, fire, TemperatureContext};
use crate::cs::PointType;
pub struct Water;
impl Water {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }
    pub fn id() -> CellType {
        2
    }
}
impl CellTrait for Water {
    fn update(
        &self,
        i: PointType,
        j: PointType,
        cur: usize,
        container: &mut [CellType],
        pal_container: &CellRegistry,
        dim: &mut Prng,
        temp_context: Option<&mut TemperatureContext>,
    ) {
        let is_falling = fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1);
        
        // Проверяем температуру во время падения в половине случаев
        if is_falling {
            if let Some(temp_ctx) = temp_context {
                // Проверяем температуру в половине случаев во время падения
                if dim.next() > 127 {
                    let temperature = (temp_ctx.get_temp)(i, j);
                    
                    // Вода испаряется в пар при высокой температуре (включая нагрев от кислоты)
                    if temperature > 15.0 {
                        use super::steam::Steam;
                        container[cur] = Steam::id();
                        // При испарении сильно поглощается тепло (сильно охлаждаем среду)
                        (temp_ctx.add_temp)(i, j + 1, -25.0); // верх
                        (temp_ctx.add_temp)(i, j - 1, -25.0); // низ
                        (temp_ctx.add_temp)(i + 1, j, -25.0); // право
                        (temp_ctx.add_temp)(i - 1, j, -25.0); // лево
                        return;
                    }
                    
                    // Вода замерзает при низкой температуре (нужна достаточно низкая температура)
                    if temperature < -3.0 {
                        container[cur] = Ice::id();
                        // Устанавливаем температуру льда около 0 градусов через add_temp
                        let target_temp = 0.0;
                        (temp_ctx.add_temp)(i, j, target_temp - temperature);
                        // При кристаллизации выделяется умеренное количество тепла
                        (temp_ctx.add_temp)(i, j + 1, 3.0); // верх
                        (temp_ctx.add_temp)(i, j - 1, 3.0); // низ
                        (temp_ctx.add_temp)(i + 1, j, 3.0); // право
                        (temp_ctx.add_temp)(i - 1, j, 3.0); // лево
                        return;
                    }
                }
            }
            return;
        }
        
        // Когда вода не падает, проверяем температуру всегда
        if !is_falling {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let arr = [top, down, l, r];
            let cc = arr[(dim.next() % 4) as usize];

            // Проверка температуры для испарения и замерзания
            if let Some(temp_ctx) = temp_context {
                let temperature = (temp_ctx.get_temp)(i, j);
                
                // Вода испаряется в пар при высокой температуре (включая нагрев от кислоты)
                if temperature > 15.0 {
                    use super::steam::Steam;
                    container[cur] = Steam::id();
                    // При испарении сильно поглощается тепло (сильно охлаждаем среду)
                    (temp_ctx.add_temp)(i, j + 1, -25.0); // верх
                    (temp_ctx.add_temp)(i, j - 1, -25.0); // низ
                    (temp_ctx.add_temp)(i + 1, j, -25.0); // право
                    (temp_ctx.add_temp)(i - 1, j, -25.0); // лево
                    return;
                }
                
                // Вода замерзает при низкой температуре (нужна достаточно низкая температура)
                // Используем более низкий порог, чтобы избежать цепной реакции
                if temperature < -3.0 {
                    container[cur] = Ice::id();
                    // Устанавливаем температуру льда около 0 градусов через add_temp
                    let target_temp = 0.0;
                    (temp_ctx.add_temp)(i, j, target_temp - temperature);
                    // При кристаллизации выделяется умеренное количество тепла
                    // Достаточно, чтобы немного нагреть среду, но не вызвать цепную реакцию
                    (temp_ctx.add_temp)(i, j + 1, 3.0); // верх
                    (temp_ctx.add_temp)(i, j - 1, 3.0); // низ
                    (temp_ctx.add_temp)(i + 1, j, 3.0); // право
                    (temp_ctx.add_temp)(i - 1, j, 3.0); // лево
                    return;
                }
            } else {
                // Проверка на замерзание при контакте со льдом (старая логика для совместимости)
                if dim.next() > 240 {
                    let top_v = container[top];
                    let down_v = container[down];
                    let r_v = container[r];
                    let l_v = container[l];

                    // Проверяем соседние клетки на наличие льда
                    if top_v == Ice::id() || down_v == Ice::id() ||
                       r_v == Ice::id() || l_v == Ice::id() {
                        // Если нет системы температуры, используем старую логику
                        container[cur] = Ice::id();
                        return;
                    }
                }
            }

            if dim.next() > 50 {
                let cc_v = container[cc] as usize;
                let cc_c = &pal_container.pal[cc_v];
                let cc_pt = cc_c.dissolve();

                if cc_pt != Void::id() {
                    container[cc] = Void::id();
                    container[cur] = cc_pt;
                    return;
                }
            }
        }
    }

    fn den(&self) -> i8 {
        1
    }

    fn name(&self) -> &str {
        "water"
    }

    fn id(&self) -> CellType {
        2
    }
}
