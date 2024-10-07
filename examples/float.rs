use std::mem;

const fn mask(bits: usize) -> u32 {
    (1 << bits) - 1
}

fn split_f32(x: f32) -> (u8, u8, u32) {
    let xb = x.to_bits();
    let s = (xb >> 31) & mask(1);
    let e = (xb >> 23) & mask(8);
    let m = (xb >> 0) & mask(23);
    (s.try_into().unwrap(), e.try_into().unwrap(), m)
}

fn make_f32(s: u8, e: u8, m: u32) -> f32 {
    let mut xb = 0u32;
    assert_eq!(s, s & 1);
    xb |= (s as u32) << 31;
    xb |= (e as u32) << 23;
    assert_eq!(m, m & mask(23));
    xb |= m;
    f32::from_bits(xb)
}

fn main() {
    println!("float");

    dbg!(split_f32(1.0));
    dbg!(split_f32(2.0));
    dbg!(split_f32(4.0));
    dbg!(split_f32(8.0));

    dbg!(make_f32(0, 127, 100));

    let two = make_f32(0, 127, 2);
    let three = make_f32(0, 127, 3);
    dbg!(two, three);
    dbg!(two * three);
    dbg!(split_f32(two * three));
}
