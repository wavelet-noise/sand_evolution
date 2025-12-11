use super::{*, helper::fluid_falling_helper, TemperatureContext};
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
        // ВАЖНО: Проверяем температуру ПЕРЕД падением, чтобы избежать дубликатов
        // Если проверять после swap, то cur уже не содержит воду!
        if let Some(temp_ctx) = temp_context {
            let temperature = (temp_ctx.get_temp)(i, j);
            
            // Вода испаряется в пар при высокой температуре
            if temperature > 15.0 {
                use super::steam::Steam;
                container[cur] = Steam::id();
                // При испарении поглощается тепло (умеренно, чтобы соседняя вода не замерзала)
                (temp_ctx.add_temp)(i, j + 1, -5.0);
                (temp_ctx.add_temp)(i, j - 1, -5.0);
                (temp_ctx.add_temp)(i + 1, j, -5.0);
                (temp_ctx.add_temp)(i - 1, j, -5.0);
                return;
            }
            
            // Вода замерзает при низкой температуре
            if temperature < -3.0 {
                use super::crushed_ice::CrushedIce;
                container[cur] = CrushedIce::id();
                return;
            }
        }
        
        // Теперь пробуем упасть (после проверки температуры)
        let is_falling = fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1);
        
        if is_falling {
            return;
        }
        
        // Когда вода не падает, проверяем растворение
        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

        let arr = [top, down, l, r];
        let cc = arr[(dim.next() % 4) as usize];

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
