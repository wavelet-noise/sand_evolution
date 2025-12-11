use crate::cs::{self, PointType};

use super::{
    helper::sand_falling_helper, void::Void, water::Water, CellRegistry, CellTrait, CellType, Prng, TemperatureContext,
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
            if temperature > 0.0 {
                // Отдаем холод соседям при таянии (уменьшено, чтобы избежать вспышек)
                (temp_ctx.add_temp)(i, j + 1, -5.0); // верх
                (temp_ctx.add_temp)(i, j - 1, -5.0); // низ
                (temp_ctx.add_temp)(i + 1, j, -5.0); // лево
                (temp_ctx.add_temp)(i - 1, j, -5.0); // право
                
                container[cur] = Water::id();
                return;
            }
        }
        
        if sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng) {
            return;
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

        let arr = [top, left, right, bot];
        let cc = arr[(prng.next() % 4) as usize];
        let rand_v = container[cc];

        if rand_v == Void::id() || rand_v == Water::id() {
            container[cur] = Water::id();
        }
    }

    fn den(&self) -> i8 {
        5
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
