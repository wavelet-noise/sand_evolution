use super::{*, helper::fluid_falling_helper};
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
    ) {
        if !fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim, 1) {
            let top = cs::xy_to_index(i, j + 1);
            let down = cs::xy_to_index(i, j - 1);
            let r = cs::xy_to_index(i + 1, j);
            let l = cs::xy_to_index(i - 1, j);

            let arr = [top, down, l, r];
            let cc = arr[(dim.next() % 4) as usize];

            // Проверка на замерзание при контакте со льдом
            // Используем высокий порог для медленного замерзания
            if dim.next() > 240 {
                let top_v = container[top];
                let down_v = container[down];
                let r_v = container[r];
                let l_v = container[l];

                // Проверяем соседние клетки на наличие льда
                if top_v == Ice::id() || down_v == Ice::id() ||
                   r_v == Ice::id() || l_v == Ice::id() {
                    // Вода замерзает и превращается в лёд (медленно)
                    container[cur] = Ice::id();
                    return;
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
