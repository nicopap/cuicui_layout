use std::marker::PhantomData;

use super::{as_usize, header, Ast, TemplateLibrary};

pub(in crate::parser) struct AstBuilder {
    pub_templates: Vec<usize>,
    ast: Vec<header::Block>,
    #[cfg(not(feature = "more_unsafe"))]
    zero_header: Vec<(usize, &'static str)>,
}
pub(in crate::parser) struct Buffer<'a, const N: usize>(pub(super) &'a mut [header::Block; N]);

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            pub_templates: Vec::new(),
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
    pub fn reserve_pub_header<T: WriteHeader>(&mut self, is_pub: bool) -> AstBuilderHead<T> {
        let index = self.ast.len();
        self.ast.extend((0..T::SIZE).map(|_| header::Block(0)));
        #[cfg(not(feature = "more_unsafe"))]
        {
            self.zero_header.push((index, std::any::type_name::<T>()));
        }
        AstBuilderHead { index, p: PhantomData, is_pub }
    }
    pub fn reserve_header<T: WriteHeader>(&mut self) -> AstBuilderHead<T> {
        self.reserve_pub_header(false)
    }
    #[allow(clippy::needless_pass_by_value)] // We want to take ownership of `head`
    pub fn write<T, const N: usize>(&mut self, head: AstBuilderHead<T>, writer: T)
    where
        T: std::fmt::Debug + for<'a> WriteHeader<Buffer<'a> = Buffer<'a, N>>,
    {
        #[cfg(not(feature = "more_unsafe"))]
        {
            self.zero_header.pop();
        }
        if head.is_pub {
            self.pub_templates.push(head.index);
        }
        let (start, end) = (head.index, head.index + as_usize(T::SIZE));
        bevy::log::trace!("{start} - {writer:?}");
        writer.write_header(Buffer((&mut self.ast[start..end]).try_into().unwrap()));
    }
    pub fn build(self) -> Result<Ast, TemplateLibrary> {
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
        let ast = Ast {
            buffer: self.ast.into(),
            pub_templates: self.pub_templates,
        };
        if ast.pub_templates.is_empty() {
            Ok(ast)
        } else {
            Err(TemplateLibrary(ast))
        }
    }
}

pub(in crate::parser) struct AstBuilderHead<T: WriteHeader> {
    index: usize,
    is_pub: bool,
    p: PhantomData<T>,
}
pub(in crate::parser) trait WriteHeader: Sized {
    const SIZE: u32;
    type Buffer<'a>;

    fn write_header(self, builder: Self::Buffer<'_>);
}
