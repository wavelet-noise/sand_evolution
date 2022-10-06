use crate::cs::{self, PointType};

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
    fn update(&self, i : PointType, j : PointType, cur : usize, container : & mut [u8], pal_container : &Palette, prng: &mut Prng);
    fn den(&self) -> i8;
    fn stat(&self) -> bool { false }
}

pub struct Void;
impl Void {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
}
impl CellTrait for Void {
    fn update(&self, i : PointType, j : PointType, cur : usize, container : & mut [u8], pal_container : &Palette, prng: &mut Prng)
    {

    }

    fn den(&self) -> i8 { 0 }
}

pub struct Sand;
impl Sand {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
}
impl CellTrait for Sand {
     fn update(&self, i : PointType, j : PointType, cur : usize, container : & mut [u8], pal_container : &Palette, prng: &mut Prng)
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
     }

     fn den(&self) -> i8 { 2 }
}

pub struct Water;
impl Water {
    pub const fn new() -> Self { Self }
    pub fn boxed() -> Box<Self> { Box::new(Self::new()) }
}
impl CellTrait for Water {
     fn update(&self, i : PointType, j : PointType, cur : usize, container : & mut [u8], pal_container : &Palette, prng: &mut Prng)
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
}
impl CellTrait for Steam {
     fn update(&self, i : PointType, j : PointType, cur : usize, container : & mut [u8], pal_container : &Palette, prng: &mut Prng)
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
}
impl CellTrait for Stone {
     fn update(&self, i : PointType, j : PointType, cur : usize, container : & mut [u8], pal_container : &Palette, prng: &mut Prng)
     {
        
     }

     fn den(&self) -> i8 { -1 }

    fn stat(&self) -> bool { true }
}