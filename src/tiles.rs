trait Packable {}

#[derive(Copy, Clone, Debug)]
#[repr(align(64))]
struct Tile<T, const LTW: usize>([u8; 64], PhantomData<T>);

type RowTile<T> = Tile<T, 4>;
type ColTile<T> = Tile<T, 0>;

impl<T, const LTW: usize> Tile<T, LTW> {
    const LTH: usize = 6 - (mem::size_of::<T>().ilog2() as usize) - LTW;
    fn zero() -> Self {
        Tile([0; 64], PhantomData)
    }
}

const _: () = {
    assert!(mem::size_of::<Tile<(), 0>>() == 64);
    assert!(mem::align_of::<Tile<(), 0>>() == 64);
};

struct Mat<T, const LTW: usize> {
    tiles: Vec<Tile<T, LTW>>,
    tiles_per_row: usize,
}

impl<T, const LTW: usize> Mat<T, LTW>
where
    T: Copy,
{
    fn load_scalar(&self, r: usize, c: usize) -> T {
        let tr = r >> Tile::<T, LTW>::LTH;
        let tc = c >> LTW;
        let t = &self.tiles[(self.tiles_per_row * tr) + tc];
        // let s = slice::from_raw_parts(t.0.as_slice(), 1 << ())

        todo!()
    }
}
