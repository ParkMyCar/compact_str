use cfg_if::cfg_if;

/// Checks if `discriminant` equals [`super::HEAP_MASK`], if so, moves `pointer_src` to
/// `pointer_dst`, and `length_src` to `length_dst`.
///
/// # Special Intrinsics
/// * `aarch64`, uses the `csel` instruction
/// * `x86` and `x86_64`, uses the `cmovz`
#[inline(always)]
pub fn cmov_ptr_len(
    discriminant: u8,
    pointer_src: *const u8,
    pointer_dst: &mut *const u8,
    length_src: usize,
    length_dst: &mut usize,
) {
    let discriminant = discriminant as usize;
    cfg_if! {
        if #[cfg(all(target_arch = "aarch64", not(miri)))] {
            unsafe {
                core::arch::asm! {
                    "cmp {d}, 254",
                    "csel {p}, {hp}, {sp}, EQ",
                    "csel {l}, {hl}, {sl}, EQ",
                    d = in(reg) discriminant,
                    p = inlateout(reg) *pointer_dst,
                    hp = in(reg) *pointer_dst,
                    sp = in(reg) pointer_src,
                    l = inlateout(reg) *length_dst,
                    hl = in(reg) *length_dst,
                    sl = in(reg) length_src,
                    options(pure, nomem, nostack),
                };
            }
        } else if #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(miri)))] {
            unsafe {
                core::arch::asm! {
                    "cmp {d}, 254",
                    "cmovnz {pd}, {ps}",
                    "cmovnz {ld}, {ls}",
                    d = in(reg) discriminant,
                    pd = inlateout(reg) *pointer_dst,
                    ps = in(reg) pointer_src,
                    ld = inlateout(reg) *length_dst,
                    ls = in(reg) length_src,
                    options(pure, nomem, nostack),
                };
            }
        } else {
            if discriminant != 254 {
                *pointer_dst = pointer_src;
                *length_dst = length_src;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::cmov_ptr_len;
    use crate::repr::HEAP_MASK;

    #[test]
    fn test_cmov_ptr_len_move() {
        let discriminant = 1;

        let a_box = Box::new(1_u8);
        let a_ptr = a_box.as_ref() as *const u8;

        let b_box = Box::new(42_u8);
        let mut b_ptr = b_box.as_ref() as *const u8;

        let a_usize = 22_usize;
        let mut b_usize = 100_usize;

        let ptr_dst = &mut b_ptr;
        let len_dst = &mut b_usize;

        cmov_ptr_len(discriminant, a_ptr, ptr_dst, a_usize, len_dst);

        // the move __should__ have occurred
        unsafe { assert_eq!(**ptr_dst, 1) };
        assert_eq!(*len_dst, 22);
    }

    #[test]
    fn test_cmov_ptr_len_no_move() {
        let discriminant = HEAP_MASK;

        let a_box = Box::new(1_u8);
        let a_ptr = a_box.as_ref() as *const u8;

        let b_box = Box::new(42_u8);
        let mut b_ptr = b_box.as_ref() as *const u8;

        let a_usize = 22_usize;
        let mut b_usize = 100_usize;

        let ptr_dst = &mut b_ptr;
        let len_dst = &mut b_usize;

        cmov_ptr_len(discriminant, a_ptr, ptr_dst, a_usize, len_dst);

        // the move should not have occurred
        unsafe { assert_eq!(**ptr_dst, 42) };
        assert_eq!(*len_dst, 100);
    }
}
