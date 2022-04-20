/// Constant-time comparison.
#[inline(always)]
fn ct_less(a: u64, b: u64) -> u64 {
    // Do not convert to Choice since it is only used internally,
    // and we don't want loss of performance.
    (a < b) as u64
}

/// Add a to the number defined by (c0,c1,c2). c2 must never overflow.
pub fn sumadd(a: u64, c0: u64, c1: u64, c2: u64) -> (u64, u64, u64) {
    let new_c0 = c0.wrapping_add(a); // overflow is handled on the next line
    let over = ct_less(new_c0, a);
    let new_c1 = c1.wrapping_add(over); // overflow is handled on the next line
    let new_c2 = c2 + ct_less(new_c1, over); // never overflows by contract
    (new_c0, new_c1, new_c2)
}

/// Add a to the number defined by (c0,c1). c1 must never overflow, c2 must be zero.
pub fn sumadd_fast(a: u64, c0: u64, c1: u64) -> (u64, u64) {
    let new_c0 = c0.wrapping_add(a); // overflow is handled on the next line
    let new_c1 = c1 + ct_less(new_c0, a); // never overflows by contract (verified the next line)
    debug_assert!((new_c1 != 0) | (new_c0 >= a));
    (new_c0, new_c1)
}

/// Add a*b to the number defined by (c0,c1,c2). c2 must never overflow.
pub fn muladd(a: u64, b: u64, c0: u64, c1: u64, c2: u64) -> (u64, u64, u64) {
    let t = (a as u128) * (b as u128);
    let th = (t >> 64) as u64; // at most 0xFFFFFFFFFFFFFFFE
    let tl = t as u64;

    let new_c0 = c0.wrapping_add(tl); // overflow is handled on the next line
    let new_th = th + if new_c0 < tl { 1 } else { 0 }; // at most 0xFFFFFFFFFFFFFFFF
    let new_c1 = c1.wrapping_add(new_th); // overflow is handled on the next line
    let new_c2 = c2 + ct_less(new_c1, new_th); // never overflows by contract (verified in the next line)
    debug_assert!((new_c1 >= new_th) || (new_c2 != 0));
    (new_c0, new_c1, new_c2)
}

/// Add a*b to the number defined by (c0,c1). c1 must never overflow.
pub fn muladd_fast(a: u64, b: u64, c0: u64, c1: u64) -> (u64, u64) {
    let t = (a as u128) * (b as u128);
    let th = (t >> 64) as u64; // at most 0xFFFFFFFFFFFFFFFE
    let tl = t as u64;

    let new_c0 = c0.wrapping_add(tl); // overflow is handled on the next line
    let new_th = th + ct_less(new_c0, tl); // at most 0xFFFFFFFFFFFFFFFF
    let new_c1 = c1 + new_th; // never overflows by contract (verified in the next line)
    debug_assert!(new_c1 >= new_th);
    (new_c0, new_c1)
}