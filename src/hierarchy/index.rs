/// The implementors can be used to index the [`super::OntologyHierarchy`].
pub trait HierarchyIdx: Copy + Ord {
    fn new(idx: usize) -> Self;
}


macro_rules! impl_idx {
    ($TYPE:ty) => {
        impl HierarchyIdx for $TYPE {
            fn new(idx: usize) -> Self {
                assert!(idx <= <$TYPE>::MAX as usize);
                idx as $TYPE
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
