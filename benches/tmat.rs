use std::arch::aarch64;

use divan::{counter::BytesCount, Bencher};
use itertools::izip;
use p3_matrix_layout_tests::tiled_mat::{TMat, Tile};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

fn main() {
    divan::main();
}

#[divan::bench(
    min_time = 1, max_time = 5,
    threads = false,
    args = [(10, 8), (18, 12)],
    consts = [0,2,4],
)]
fn fold_rows_u32_sum<const LTW: usize>(b: Bencher, (log_h, log_w): (usize, usize)) {
    let mut rng = ChaChaRng::seed_from_u64(0);
    let m = TMat::<LTW>::from_fn(1 << log_h, 1 << log_w, |_, _| rng.gen());

    b.counter(BytesCount::new(m.bytes()))
        .with_inputs(|| m.clone())
        .bench_local_refs(|m| {
            m.fold_rows(
                |_| Tile::<LTW>::zero(),
                |mut acc, tile| {
                    for (l, r) in izip!(acc.vecs_mut(), tile.vecs()) {
                        *l = unsafe { aarch64::vpaddq_u32(*l, *r) };
                        // *l = unsafe { aarch64::veorq_u32(*l, *r) };
                    }
                    acc
                },
            );
        });
}
