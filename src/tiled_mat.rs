use std::{
    arch::aarch64::uint32x4_t,
    array, cmp, fmt, iter,
    mem::{self, transmute},
    ops::Range,
};

use itertools::iproduct;
use rayon::prelude::*;

#[derive(Copy, Clone, Debug)]
#[repr(C, align(64))]
pub struct Tile<const LTW: usize>([u32; 16]);

const fn mask(bits: usize) -> usize {
    (1 << bits) - 1
}

impl<const LTW: usize> Tile<LTW> {
    const LTH: usize = 4 - LTW;
    pub fn from_fn(mut f: impl FnMut(usize, usize) -> u32) -> Self {
        Self(array::from_fn(|i| f(i >> LTW, i & mask(LTW))))
    }
    pub fn zero() -> Self {
        Tile([0; 16])
    }

    /// if you don't care about arrangement
    pub fn vecs(&self) -> &[uint32x4_t; 4] {
        unsafe { &*(&self.0 as *const [u32; 16] as *const [uint32x4_t; 4]) }
    }
    pub fn vecs_mut(&mut self) -> &mut [uint32x4_t; 4] {
        unsafe { &mut *(&mut self.0 as *mut [u32; 16] as *mut [uint32x4_t; 4]) }
    }
}

#[derive(Clone)]
pub struct TMat<const LTW: usize> {
    pub width: usize,
    pub tiles: Vec<Tile<LTW>>,
}

impl<const LTW: usize> TMat<LTW> {
    const fn tiles_per_row(&self) -> usize {
        self.width >> LTW
    }

    pub fn bytes(&self) -> usize {
        self.tiles.len() * mem::size_of::<Tile<LTW>>()
    }

    #[allow(non_snake_case)]
    pub fn from_fn(height: usize, width: usize, mut f: impl FnMut(usize, usize) -> u32) -> Self {
        let LTH = Tile::<LTW>::LTH;
        Self {
            width,
            tiles: iproduct!(0..(height >> LTH), 0..(width >> LTW))
                .map(|(tr, tc)| Tile::from_fn(|rit, cit| f((tr << LTH) + rit, (tc << LTW) + cit)))
                .collect(),
        }
    }

    pub fn fold_rows<Acc, Init, Op>(&self, init: Init, op: Op) -> Vec<Acc>
    where
        Acc: Send + Sync,
        Init: Fn(Range<usize>) -> Acc + Send + Sync,
        Op: Fn(Acc, &Tile<LTW>) -> Acc + Send + Sync,
    {
        let tpr = self.tiles_per_row();
        self.tiles
            .par_chunks_exact(tpr)
            .enumerate()
            .map(|(tr, tile_row)| {
                let mut acc = init(tr * tpr..(tr + 1) * tpr);
                for t in tile_row {
                    acc = op(acc, t);
                }
                acc
            })
            .collect()
    }

    pub fn par_row_tiles_native(&self) -> impl IndexedParallelIterator<Item = &[Tile<LTW>]> {
        let tpr = self.tiles_per_row();
        self.tiles.par_chunks_exact(tpr)
    }

    pub fn par_row_tiles_native_mut(
        &mut self,
    ) -> impl IndexedParallelIterator<Item = &mut [Tile<LTW>]> {
        let tpr = self.tiles_per_row();
        self.tiles.par_chunks_exact_mut(tpr)
    }

    pub fn par_row_tiles<const O_LTW: usize>(
        &self,
    ) -> impl IndexedParallelIterator<Item = TileIter<'_, LTW, O_LTW>> {
        let tile_rows_per_iter = 1 << LTW.saturating_sub(O_LTW);
        let tpr = self.tiles_per_row();
        self.tiles
            .par_chunks_exact(tile_rows_per_iter * tpr)
            .map(move |chunk| TileIter {
                idx: 0,
                chunk,
                tile_rows: tile_rows_per_iter,
            })
    }

    /*
    pub fn zero(height: usize, width: usize) -> Self {
        Self::from_fn(height, width, |_, _| 0)
    }
    fn tile_row_mut(&mut self, tr: usize) -> &mut [Tile] {
        let tpr = self.tiles_per_row();
        &mut self.tiles[(tr * tpr)..((tr + 1) * tpr)]
    }
    pub fn get_mut(&mut self, r: usize, c: usize) -> &mut u32 {
        let (tr, rit) = (r >> Self::LTH, r & mask(Self::LTH));
        let (tc, cit) = (c >> LTW, c & mask(LTW));

        let tpr = self.tiles_per_row();
        let tile = &mut self.tiles[tpr * tr + tc];

        &mut tile.0[(rit << LTW) + cit]
    }
    */
}

pub struct TileIter<'t, const I_LTW: usize, const O_LTW: usize> {
    idx: usize,
    chunk: &'t [Tile<I_LTW>],
    tile_rows: usize,
}

impl<'t, const I_LTW: usize, const O_LTW: usize> Iterator for TileIter<'t, I_LTW, O_LTW> {
    type Item = Tile<O_LTW>;
    fn next(&mut self) -> Option<Self::Item> {
        let o_ltw = 1 << O_LTW;
        let i_ltw = 1 << I_LTW;

        let t = Tile::from_fn(|rit, cit| {
            //
            0
        });
        self.idx += 1;
        Some(t)
    }
}

impl<const LTW: usize> fmt::Debug for TMat<LTW> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for tile_row in self.tiles.chunks_exact(self.tiles_per_row()) {
            for tile in tile_row {
                write!(f, "[")?;
                for elt in tile.0 {
                    write!(f, " {elt:#010x}")?;
                }
                write!(f, "]")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let (log_h, log_w) = (4, 4);
        let m = TMat::<3>::from_fn(1 << log_h, 1 << log_w, |r, c| ((r << log_w) + c) as u32);
        dbg!(&m);

        // assert_eq!(1, 2);
    }
}
