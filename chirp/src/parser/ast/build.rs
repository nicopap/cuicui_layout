use std::marker::PhantomData;

use super::{as_usize, header, Ast};

pub(in crate::parser) struct AstBuilder {
    ast: Vec<header::Block>,
    #[cfg(not(feature = "more_unsafe"))]
    zero_header: Vec<(usize, &'static str)>,
}
pub(in crate::parser) struct Buffer<'a, const N: usize>(pub(super) &'a mut [header::Block; N]);

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            ast: Vec::with_capacity(128),
            #[cfg(not(feature = "more_unsafe"))]
            zero_header: Vec::new(),
        }
    }
    pub fn write_header<T, const N: usize>(&mut self, writer: T)
    where
        T: std::fmt::Debug + for<'a> WriteHeader<Buffer<'a> = Buffer<'a, N>>,
    {
        let header = self.reserve_header::<T>();
        self.write(header, writer);
    }
    pub fn reserve_header<T: WriteHeader>(&mut self) -> AstBuilderHead<T> {
        let index = self.ast.len();
        self.ast.extend((0..T::SIZE).map(|_| header::Block(0)));
        #[cfg(not(feature = "more_unsafe"))]
        {
            self.zero_header.push((index, std::any::type_name::<T>()));
        }
        AstBuilderHead { index, p: PhantomData }
    }
    #[cfg_attr(feature = "more_unsafe", allow(unused_mut))]
    pub fn write<T, const N: usize>(&mut self, mut head: AstBuilderHead<T>, writer: T)
    where
        T: std::fmt::Debug + for<'a> WriteHeader<Buffer<'a> = Buffer<'a, N>>,
    {
        #[cfg(not(feature = "more_unsafe"))]
        {
            self.zero_header.pop();
        }
        let (start, end) = (head.index, head.index + as_usize(T::SIZE));
        bevy::log::trace!("{start} - {writer:?}");
        writer.write_header(Buffer((&mut self.ast[start..end]).try_into().unwrap()));
    }
    pub fn build(self) -> Ast {
        #[cfg(not(feature = "more_unsafe"))]
        {
            for (index, name) in &self.zero_header {
                bevy::log::error!(
                    "{index} - {name}: Created header that never was initialized. \
                        This is a cuicui_chirp bug, please open an issue at\n\n\
                        https://github.com/nicopap/cuicui_layout/issues/new\n",
                );
            }
            assert!(
                self.zero_header.is_empty(),
                "Proceeding would be unsound, aborting.."
            );
        }
        Ast(self.ast.into())
    }
}

pub(in crate::parser) struct AstBuilderHead<T: WriteHeader> {
    index: usize,
    p: PhantomData<T>,
}

pub(in crate::parser) trait WriteHeader: Sized {
    const SIZE: u32;
    type Buffer<'a>;

    fn write_header(self, builder: Self::Buffer<'_>);
}
