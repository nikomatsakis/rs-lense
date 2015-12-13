use std::{ops, slice};

fn div_up(n: usize, m: usize) -> usize {
    if n % m == 0 {
        n / m
    } else {
        n / m + 1
    }
}

/// 8-byte aligned Deref<Target=[u8]>
pub struct Aligned {
    vec: Vec<u64>,
    capacity: usize,
}

impl Aligned {
    pub fn new(c: usize) -> Self {
        Aligned::from_vec(vec![0; div_up(c, 8)], c)
    }

    pub fn from_vec(v: Vec<u64>, c: usize) -> Self {
        Aligned { vec: v, capacity: c }
    }
}

impl ops::Deref for Aligned {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.vec.as_ptr() as *const u8, self.capacity)
        }
    }
}

impl ops::DerefMut for Aligned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.vec.as_mut_ptr() as *mut u8, self.capacity)
        }
    }
}
