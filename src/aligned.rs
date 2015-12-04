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
    len: usize,
}

impl Aligned {
    pub fn new(cap: usize) -> Self {
        Aligned {
            vec: vec![0; div_up(cap, 8)],
            len: cap,
        }
    }
}

impl ops::Deref for Aligned {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.vec.as_ptr() as *const u8, self.len)
        }
    }
}

impl ops::DerefMut for Aligned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.vec.as_mut_ptr() as *mut u8, self.len)
        }
    }
}

#[test]
fn size() {
    let v = Aligned::new(9);
    assert_eq!(v.len(), 9);
    assert_eq!(v.vec.len(), 2);
}
