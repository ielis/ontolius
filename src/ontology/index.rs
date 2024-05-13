//! Ontology term indexers.

/// The implementors can be used to index the [`super::TermAware`].
pub trait TermIdx: Copy {

    // Convert the index to `usize` for indexing.
    fn index(self) -> usize;

}

macro_rules! impl_idx {
    ($TYPE:ty) => {
        impl TermIdx for $TYPE {
            fn index(self) -> usize {
                self as usize
            }
        }
    }
}

impl_idx!(u8);
impl_idx!(u16);
impl_idx!(u32);
impl_idx!(u64);
impl_idx!(usize);

impl_idx!(i8);
impl_idx!(i16);
impl_idx!(i32);
impl_idx!(i64);
impl_idx!(isize);
