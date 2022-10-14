mod helper;
use crate::cs::{self, PointType};

use self::helper::sand_faling_helper;
pub type CellType = u8;

pub struct Prng {
    rnd: [u8; 256],
    rnd_next: usize
}

impl Prng {
    pub fn new() -> Self { 
        let mut buf = [0u8; 256];
        _ = getrandom::getrandom(&mut buf);

         Self { rnd: buf, rnd_next: 0 }
    }
    pub fn next(&mut self) -> u8 { 
        self.rnd_next += 1; 
        self.rnd_next = if self.rnd_next >= 256 { 0 } else {  self.rnd_next };

        return self.rnd[self.rnd_next];
    }
}

pub struct Palette {
    pub pal: Vec<Box<dyn CellTrait>>,
}

impl Palette {
    pub const fn new() -> Self { Self { pal: Vec::new() } }
}

pub trait CellTrait {
    fn update(&self, i: PointType, j: PointType, cur : usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Prng);
    fn den(&self) -> i8 { 0 }
    fn stat(&self) -> bool { false }
    fn burnable(&self) -> CellType { Void::id() }
    fn heatable(&self) -> CellType { Void::id() }
}

pub struct Void;
impl Void {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 0 }
}
impl CellTrait for Void {
    fn update(&self, _: PointType, _: PointType, _: usize, _: & mut [u8], _: &Palette, _: &mut Prng)
    {
        
    }
}

pub struct Sand;
impl Sand {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 1 }
}
impl CellTrait for Sand {
     fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, _: &mut Prng)
     {
        sand_faling_helper(self.den(), i, j, container, pal_container, cur);
     }

     fn den(&self) -> i8 { 2 }
}

pub struct Water;
impl Water {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 2 }
}

impl CellTrait for Water {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Prng)
    {
        let down = cs::xy_to_index(i, j - 1);
        let down_v = container[down] as usize;
        let down_c = &pal_container.pal[down_v];

        let dl = cs::xy_to_index(i - 1, j - 1);
        let dl_v = container[dl] as usize;
        let dl_c = &pal_container.pal[dl_v];

        let dr = cs::xy_to_index(i + 1, j - 1);
        let dr_v = container[dr] as usize;
        let dr_c = &pal_container.pal[dr_v];

        let r = cs::xy_to_index(i + 1, j);
        let r_v = container[r] as usize;
        let r_c = &pal_container.pal[r_v];

        let l = cs::xy_to_index(i - 1, j);
        let l_v = container[l] as usize;
        let l_c = &pal_container.pal[l_v];

        if down_c.den() < self.den() && !down_c.stat()
        {
            container.swap(cur, down);
        }
        else if dr_c.den() < self.den() && !dr_c.stat()
        {
            container.swap(cur, dr);
        }
        else if dl_c.den() < self.den() && !dl_c.stat()
        {
            container.swap(cur, dl);
        }
        else if r_c.den() < self.den() && !r_c.stat()
        {
            container.swap(cur, r);
        }
        else if l_c.den() < self.den() && !l_c.stat()
        {
            container.swap(cur, l);
        }
        else if prng.next() == 0 && prng.next() == 0
        {
            container[cur] = 3;
        }
    }

    fn den(&self) -> i8 { 1 }
}

pub struct Steam;
impl Steam {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 3 }
}
impl CellTrait for Steam {
    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Prng)
    {
        let down = cs::xy_to_index(i, j + 1);
        let down_v = container[down] as usize;
        let down_c = &pal_container.pal[down_v];

        let dl = cs::xy_to_index(i - 1, j + 1);
        let dl_v = container[dl] as usize;
        let dl_c = &pal_container.pal[dl_v];

        let dr = cs::xy_to_index(i + 1, j + 1);
        let dr_v = container[dr] as usize;
        let dr_c = &pal_container.pal[dr_v];

        let r = cs::xy_to_index(i + 1, j);
        let r_v = container[r] as usize;
        let r_c = &pal_container.pal[r_v];

        let l = cs::xy_to_index(i - 1, j);
        let l_v = container[l] as usize;
        let l_c = &pal_container.pal[l_v];

        if down_c.den() > self.den() && !down_c.stat()
        {
            container.swap(cur, down);
        }
        else if dr_c.den() > self.den() && !dr_c.stat()
        {
            container.swap(cur, dr);
        }
        else if dl_c.den() > self.den() && !dl_c.stat()
        {
            container.swap(cur, dl);
        }
        else if r_c.den() > self.den() && !r_c.stat()
        {
            container.swap(cur, r);
        }
        else if l_c.den() > self.den() && !l_c.stat()
        {
            container.swap(cur, l);
        }
        else if prng.next() == 0 && prng.next() == 0
        {
            container[cur] = 2;
        }
    }

    fn den(&self) -> i8 { -1 }
}

pub struct Stone;
impl Stone {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 255 }
}
impl CellTrait for Stone {
    fn update(&self, _: PointType, _: PointType, _ : usize, _: & mut [CellType], _: &Palette, _: &mut Prng)
    {
    
    }

    fn stat(&self) -> bool { true }
}

pub struct Fire;
impl Fire {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 4 }
}
impl CellTrait for Fire {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Prng)
    {
        if prng.next() > 128
        {
		    return;
        }

        if prng.next() > 200
        {
            container[cur] = 0;
		    return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let down = cs::xy_to_index(i, j - 1);
        let r = cs::xy_to_index(i + 1, j);
        let l = cs::xy_to_index(i - 1, j);

	    let arr = [top, down, l, r];
	    let cc = arr[(prng.next() % 4) as usize];

        let cc_v = container[cc] as usize;
        let cc_c = &pal_container.pal[cc_v];
        let cc_b = cc_c.burnable();

        if cc_b != Void::id()
        {
            container[cc] = cc_b;
            return;
        }

        let cc_h = cc_c.heatable();

        if cc_h != Void::id()
        {
            container[cc] = cc_h;
            return;
        }

        let top_v = container[top];

        if top_v == Void::id()
        {
            container.swap(cur, top);
            return;
        }

        let topl = cs::xy_to_index(i - 1, j + 1);
        let topl_v = container[topl];

        if topl_v == Void::id()
        {
            container.swap(cur, topl);
            return;
        }

        let topr = cs::xy_to_index(i + 1, j + 1);
        let topr_v = container[topr];

        if topr_v == Void::id()
        {
            container.swap(cur, topr);
            return;
        }

        container[cur] = 0;
    }

    fn den(&self) -> i8 { -1 }
}

pub struct Wood;
impl Wood {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 5 }
}
impl CellTrait for Wood {

    fn update(&self, _: PointType, _: PointType, _: usize, _: & mut [CellType], _: &Palette, _: &mut Prng)
    {
        
    }

    fn stat(&self) -> bool { true }
    fn burnable(&self) -> u8 { BurningWood::id() }
}

pub struct BurningWood;
impl BurningWood {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 6 }
}
impl CellTrait for BurningWood {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Prng)
    {
        if prng.next() > 200
        {
		    return;
        }

        if prng.next() > 250
        {
            container[cur] = BurningCoal::id();
		    return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let topl = cs::xy_to_index(i - 1, j + 1);
        let topr = cs::xy_to_index(i + 1, j + 1);
        
	    let arr = [top, topl, topr];
	    let cc = arr[(prng.next() % 3) as usize];
        let top_v = container[cc];

        if top_v == Void::id() {
            container[cc] = Fire::id();
        }
    }

    fn stat(&self) -> bool { true }
}

pub struct BurningCoal;
impl BurningCoal {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
    pub fn id() -> CellType { 7 }
}
impl CellTrait for BurningCoal {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Prng)
    {
        if !sand_faling_helper(self.den(), i, j, container, pal_container, cur) {
            if prng.next() > 200
            {
                return;
            }

            if prng.next() > 250
            {
                container[cur] = Void::id();
                return;
            }

            let top = cs::xy_to_index(i, j + 1);
            let topl = cs::xy_to_index(i - 1, j + 1);
            let topr = cs::xy_to_index(i + 1, j + 1);
            
            let arr = [top, topl, topr];
            let cc = arr[(prng.next() % 3) as usize];
            let top_v = container[cc];

            if top_v == Void::id() {
                container[cc] = Fire::id();
            }

            let bot = cs::xy_to_index(i, j - 1);
            let bot_v = container[bot] as usize;
            let bot_c = &pal_container.pal[bot_v];
            let bot_b = bot_c.burnable();

            if bot_b != Void::id()
            {
                container[bot] = bot_b;
                return;
            }
        }
    }

    fn den(&self) -> i8 { 2 }
}