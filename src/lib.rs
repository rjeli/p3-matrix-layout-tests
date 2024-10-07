/*

benches to run:
- colwise_dot_product
- dot_ext_powers
- fft ??

- something simple like iterating over all values horiz, and vert

*/

#![allow(dead_code)]

use itertools::{izip, Itertools};
use p3_circle::{CircleDomain, Point};
use p3_field::{batch_multiplicative_inverse, extension::ComplexExtendable, AbstractField, Field};
use p3_mersenne_31::Mersenne31;
use p3_util::reverse_slice_index_bits;
use rand::Rng;

mod tinym31;

pub mod tiled_mat;

type F = Mersenne31;

fn compute_twiddles(domain: CircleDomain<F>) -> Vec<Vec<F>> {
    assert!(domain.log_n >= 1);
    let mut pts = domain.coset0().collect_vec();
    reverse_slice_index_bits(&mut pts);
    let mut twiddles = vec![pts.iter().map(|p| p.y).collect_vec()];
    if domain.log_n >= 2 {
        twiddles.push(pts.iter().step_by(2).map(|p| p.x).collect_vec());
        for i in 0..(domain.log_n - 2) {
            let prev = twiddles.last().unwrap();
            assert_eq!(prev.len(), 1 << (domain.log_n - 2 - i));
            let cur = prev
                .iter()
                .step_by(2)
                .map(|x| x.square().double() - F::one())
                .collect_vec();
            twiddles.push(cur);
        }
    }
    twiddles
}

fn circle_basis(p: Point<F>, log_n: usize) -> Vec<F> {
    let mut b = vec![F::one(), p.y];
    let mut x = p.x;
    for _ in 0..(log_n - 1) {
        for i in 0..b.len() {
            b.push(b[i] * x);
        }
        x = x.square().double() - F::one();
    }
    assert_eq!(b.len(), 1 << log_n);
    reverse_slice_index_bits(&mut b);
    b
}

/*
fn deinterleaved(index: usize, log_n: usize) -> usize {
    let (index, lsb) = (index >> 1, index & 1);
    if lsb == 0 {
        index
    } else {
        (1 << log_n) - index - 1
    }
}
fn interleaved(index: usize, log_n: usize) -> usize {
    let (index, msb) = (index & ((1 << (log_n - 1)) - 1), index >> (log_n - 1));
    if msb == 0 {
        index
    } else {
        (1 << log_n) - index - 1
    }
}
*/

fn dif(t: F, lo: F, hi: F) -> (F, F) {
    (lo + hi, t * (lo - hi))
}

fn dit(t: F, lo: F, hi: F) -> (F, F) {
    (lo + t * hi, lo - t * hi)
}

fn interp_simple(xs: Vec<F>, twiddles: Vec<Vec<F>>) -> Vec<F> {
    // de-interleave
    let (mut xs, mut hi): (Vec<F>, Vec<F>) = xs.into_iter().tuples().unzip();
    hi.reverse();
    xs.append(&mut hi);

    for mut ts in twiddles {
        // let mut ts = batch_multiplicative_inverse(&ts);

        ts = batch_multiplicative_inverse(&ts);
        reverse_slice_index_bits(&mut ts);

        for blk in xs.chunks_exact_mut(ts.len() * 2) {
            let (los, his) = blk.split_at_mut(ts.len());
            for (&t, lo, hi) in izip!(&ts, los, his) {
                (*lo, *hi) = dif(t, *lo, *hi);
            }
        }

        /*
        let blk_sz = xs.len() / ts.len();
        for (t, blk) in izip!(ts, xs.chunks_exact_mut(blk_sz)) {
            let (los, his) = blk.split_at_mut(blk_sz / 2);
            for (lo, hi) in izip!(los, his) {
                (*lo, *hi) = dif(t, *lo, *hi);
            }
        }
        */
    }

    let h = xs.len();
    for x in &mut xs {
        *x *= F::from_canonical_usize(h).inverse();
    }

    xs
}

#[cfg(test)]
mod tests {
    use super::*;
    use p3_circle::{CircleDomain, CircleEvaluations};
    use p3_field::dot_product;
    use p3_matrix::{dense::RowMajorMatrix, Matrix};
    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;

    #[test]
    fn it_works() {
        let mut rng = ChaChaRng::seed_from_u64(0);

        let log_n = 4;
        let n = 1 << log_n;

        let d = CircleDomain::<F>::standard(log_n);
        let twiddles = compute_twiddles(d);

        /*
        dbg!(&twiddles);
        dbg!(&twiddles
            .iter()
            .map(|t| t.iter().map(|&x| -x).collect_vec())
            .collect_vec());
            */

        let evals: Vec<F> = (0..n).map(|_| rng.gen()).collect();
        let coeffs = interp_simple(evals.clone(), twiddles.clone());
        dbg!(&coeffs);

        let ref_coeffs =
            CircleEvaluations::from_natural_order(d, RowMajorMatrix::new(evals.clone(), 1))
                .interpolate();
        dbg!(ref_coeffs.to_row_major_matrix().values);

        for (i, pt) in d.points().enumerate() {
            let eval2 = dot_product(
                circle_basis(pt, log_n).iter().copied(),
                coeffs.iter().copied(),
            );
            assert_eq!(evals[i], eval2);
        }
    }
}
