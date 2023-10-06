use std::marker::PhantomData;

use super::{as_usize, header, Ast};

pub(in crate::parser) struct AstBuilder(Vec<header::Block>);
pub(in crate::parser) struct Buffer<'a, const N: usize>(pub(super) &'a mut [header::Block; N]);

impl AstBuilder {
    pub fn new() -> Self {
        Self(Vec::with_capacity(128))
    }
    pub fn write_header<T, const N: usize>(&mut self, writer: T)
    where
        T: std::fmt::Debug + for<'a> WriteHeader<Buffer<'a> = Buffer<'a, N>>,
    {
        let header = self.reserve_header::<T>();
        self.write(header, writer);
    }
    pub fn reserve_header<T: WriteHeader>(&mut self) -> AstBuilderHead<T> {
        let index = self.0.len();
        self.0.extend((0..T::SIZE).map(|_| header::Block(0)));
        AstBuilderHead {
            index,
            p: PhantomData,
            #[cfg(debug_assertions)]
            written: false,
        }
    }
    #[cfg_attr(not(debug_assertions), allow(unused_mut))]
    pub fn write<T, const N: usize>(&mut self, mut head: AstBuilderHead<T>, writer: T)
    where
        T: std::fmt::Debug + for<'a> WriteHeader<Buffer<'a> = Buffer<'a, N>>,
    {
        #[cfg(debug_assertions)]
        {
            head.written = true;
        }
        let (start, end) = (head.index, head.index + as_usize(T::SIZE));
        bevy::log::trace!("{start} - {writer:?}");
        writer.write_header(Buffer((&mut self.0[start..end]).try_into().unwrap()));
    }
    pub fn build(self) -> Ast {
        Ast(self.0.into())
    }
}

pub(in crate::parser) struct AstBuilderHead<T: WriteHeader> {
    index: usize,
    p: PhantomData<T>,
    #[cfg(debug_assertions)]
    written: bool,
}

#[cfg(debug_assertions)]
impl<T: WriteHeader> Drop for AstBuilderHead<T> {
    fn drop(&mut self) {
        let index = self.index;
        let name = bevy::utils::get_short_name(std::any::type_name::<T>());
        assert!(
            self.written,
            "{index} - {name}: Created header that never was initialized. \
                This is a cuicui_chirp bug, please open an issue at\n\n\
                https://github.com/nicopap/cuicui_layout/issues/new\n",
        );
    }
}
pub(in crate::parser) trait WriteHeader: Sized {
    const SIZE: u32;
    type Buffer<'a>;

    fn write_header(self, builder: Self::Buffer<'_>);
}
