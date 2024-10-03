/*

benches to run:
- colwise_dot_product
- dot_ext_powers
- fft ??

- something simple like iterating over all values horiz, and vert

*/

use core::{mem, slice};
use itertools::izip;
use rand::Rng;
use std::marker::PhantomData;

mod tinym31;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaChaRng;

    #[test]
    fn it_works() {
        let mut _rng = ChaChaRng::seed_from_u64(0);
    }
}
