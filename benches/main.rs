use core::{
    arch::aarch64::{self, uint32x4_t, uint32x4x4_t},
    mem::{self, transmute},
};

use divan::{counter::BytesCount, Bencher};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use rayon::prelude::*;

#[derive(Copy, Clone, Debug)]
#[repr(align(64))]
struct Tile(uint32x4x4_t);

impl Tile {
    fn broadcast(v: u32) -> Self {
        unsafe { transmute([v; 16]) }
    }
}

const _: () = {
    assert!(mem::size_of::<Tile>() == 64);
    assert!(mem::align_of::<Tile>() == 64);
};

fn main() {
    divan::main();
}

const WIDTH: usize = 4;

const P: u32 = 0x7fffffff;
const PP: uint32x4_t = unsafe { transmute::<[u32; WIDTH], _>([P; WIDTH]) };

fn m31_mul(lhs: uint32x4_t, rhs: uint32x4_t) -> uint32x4_t {
    unsafe {
        let prod_hi31 = aarch64::vreinterpretq_u32_s32(aarch64::vqdmulhq_s32(
            aarch64::vreinterpretq_s32_u32(lhs),
            aarch64::vreinterpretq_s32_u32(rhs),
        ));
        let prod_lo32 = aarch64::vmulq_u32(lhs, rhs);

        // t = lo32 - hi31 * P
        let t = aarch64::vmlsq_u32(prod_lo32, prod_hi31, PP);

        // reduce t from [0,2P] to [0,P]
        let u = aarch64::vsubq_u32(t, PP);
        aarch64::vminq_u32(t, u)
    }
}

fn m31_mul_tile(lhs: &mut Tile, rhs: &Tile) {
    // let mut lhs = Tile(unsafe { aarch64::vld1q_u32_x4(lhs_ as *const _ as *const u32) });
    // let rhs = Tile(unsafe { aarch64::vld1q_u32_x4(rhs_ as *const _ as *const u32) });

    lhs.0 .0 = m31_mul(lhs.0 .0, rhs.0 .0);
    lhs.0 .1 = m31_mul(lhs.0 .1, rhs.0 .1);
    lhs.0 .2 = m31_mul(lhs.0 .2, rhs.0 .2);
    lhs.0 .3 = m31_mul(lhs.0 .3, rhs.0 .3);

    /*
    lhs.0 .0 = m31_mul(lhs.0 .0, rhs.0 .0);
    lhs.0 .1 = m31_mul(lhs.0 .1, rhs.0 .1);
    lhs.0 .2 = m31_mul(lhs.0 .2, rhs.0 .2);
    lhs.0 .3 = m31_mul(lhs.0 .3, rhs.0 .3);

    lhs.0 .0 = m31_mul(lhs.0 .0, rhs.0 .0);
    lhs.0 .1 = m31_mul(lhs.0 .1, rhs.0 .1);
    lhs.0 .2 = m31_mul(lhs.0 .2, rhs.0 .2);
    lhs.0 .3 = m31_mul(lhs.0 .3, rhs.0 .3);
    */

    // *lhs_ = lhs;
    /*
    .25 cyc / ix
    16 b / ix
    3 gcyc / s
    ? = gb / s

    ? = gb / 3 gcyc
      = b / 3 cyc

    */
}

#[divan::bench(
    min_time = 1, max_time = 5,
    threads = false,
    args = [(10, 8), (16, 12)],
)]
fn layout_rows_op_rows(b: Bencher, (log_h, log_w): (usize, usize)) {
    let mut rng = ChaChaRng::seed_from_u64(0);

    let log_elts_per_tile =
        (mem::size_of::<Tile>().ilog2() - mem::size_of::<u32>().ilog2()) as usize;

    let log_tw = log_w - log_elts_per_tile;
    let log_th = log_h;

    let tw = 1 << log_tw;
    let th = 1 << log_th;

    let buf: Vec<Tile> = (0..(1 << (log_tw + log_th)))
        .map(|_| unsafe { transmute(rng.gen::<[u32; 16]>()) })
        .collect();

    let bytes = buf.len() * mem::size_of::<Tile>();

    b.counter(BytesCount::new(bytes))
        .with_inputs(|| buf.clone())
        .bench_refs(|buf| {
            let mut acc = vec![Tile::broadcast(1); 1 << log_tw];
            for tr in 0..th {
                for tc in 0..tw {
                    m31_mul_tile(&mut acc[tc], &buf[tw * tr + tc]);
                }
            }
            acc
        });
}

#[divan::bench(
    min_time = 1, max_time = 5,
    threads = false,
    args = [(10, 8), (16, 12)],
)]
fn layout_rows_op_rows_par(b: Bencher, (log_h, log_w): (usize, usize)) {
    let mut rng = ChaChaRng::seed_from_u64(0);

    let log_elts_per_tile =
        (mem::size_of::<Tile>().ilog2() - mem::size_of::<u32>().ilog2()) as usize;

    let log_tw = log_w - log_elts_per_tile;
    let log_th = log_h;

    let tw = 1 << log_tw;
    let th = 1 << log_th;

    let buf: Vec<Tile> = (0..(1 << (log_tw + log_th)))
        .map(|_| unsafe { transmute(rng.gen::<[u32; 16]>()) })
        .collect();

    let bytes = buf.len() * mem::size_of::<Tile>();

    b.counter(BytesCount::new(bytes))
        .with_inputs(|| buf.clone())
        .bench_refs(|buf| {
            buf.par_chunks_exact(tw)
                .fold(
                    || vec![Tile::broadcast(1); 1 << log_tw],
                    |mut acc, row| {
                        for tc in 0..tw {
                            m31_mul_tile(&mut acc[tc], &row[tc]);
                        }
                        acc
                    },
                )
                .reduce(
                    || vec![Tile::broadcast(1); 1 << log_tw],
                    |mut l, r| {
                        for tc in 0..tw {
                            m31_mul_tile(&mut l[tc], &r[tc]);
                        }
                        l
                    },
                )
        });
}
