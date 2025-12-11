use super::{*, TemperatureContext};
use crate::cs::{self, PointType};

pub struct BurningCoal;
impl BurningCoal {
    pub const fn new() -> Self {
        Self
    }
    pub fn boxed() -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn id() -> CellType {
        7
    }
}

impl CellTrait for BurningCoal {
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
        // Горящий уголь выделяет тепло
        if let Some(temp_ctx) = temp_context {
            (temp_ctx.add_temp)(i, j + 1, 3.0); // верх
            (temp_ctx.add_temp)(i, j - 1, 3.0); // низ
            (temp_ctx.add_temp)(i + 1, j, 3.0); // право
            (temp_ctx.add_temp)(i - 1, j, 3.0); // лево
        }
        if !sand_falling_helper(self.den(), i, j, container, pal_container, cur, prng) {
            let bot = cs::xy_to_index(i, j - 1);
            let bot_v = container[bot] as usize;

            let top = cs::xy_to_index(i, j + 1);

            if prng.next() > 200 {
                return;
            }

            if container[top] == Water::id() {
                container[top] = Steam::id();
                if prng.next() > 200 {
                    container[cur] = Coal::id();
                }
                return;
            }

            if prng.next() > 250 {
                container[cur] = Void::id();
                //prng.add_carb();
                return;
            }

            let topl = cs::xy_to_index(i - 1, j + 1);
            let topr = cs::xy_to_index(i + 1, j + 1);

            let arr = [top, topl, topr];
            let cc = arr[(prng.next() % 3) as usize];
            let top_v = container[cc];

            if top_v == Void::id() {
                container[cc] = fire::id();
            }

            if prng.next() > 50 {
                return;
            }

            let bot_c = &pal_container.pal[bot_v];
            let bot_b = bot_c.burnable();

            if bot_b != Void::id() {
                container[bot] = bot_b;
                return;
            }
        }
    }

    fn den(&self) -> i8 {
        2
    }
    fn proton_transfer(&self) -> CellType {
        BurningGas::id()
    }
    fn name(&self) -> &str {
        "burning coal"
    }

    fn id(&self) -> CellType {
        7
    }
}
