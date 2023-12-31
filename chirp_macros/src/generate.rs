use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{meta::ParseNestedMeta, punctuated::Punctuated, spanned::Spanned};

#[derive(Default, Debug, PartialEq)]
enum FnConfig {
    #[default]
    Method,
    Ignore,
}
#[allow(clippy::trivially_copy_pass_by_ref)] // false positive. Type necessary to avoid eta-expension
fn is_parse_dsl_attr(attr: &&syn::Attribute) -> bool {
    let result = FnConfig::default().parse_attr(attr);
    result.is_err() || result.is_ok_and(|s| s)
}
impl FnConfig {
    fn parse_attr(&mut self, attr: &syn::Attribute) -> syn::Result<bool> {
        let mut ret = false;
        if attr.path().is_ident("parse_dsl") {
            attr.parse_nested_meta(|m| self.parse(m))?;
            ret = true;
        // A workaround the fact that `cfg` attributes are not expanded before
        // attribute proc macros are ran. Yet we still need to find parse_dsl,
        // enact them and remove them, otherwise we run in compilation issues.
        // See <https://github.com/rust-lang/rust/issues/82679>
        } else if attr.path().is_ident("cfg_attr") {
            let parser = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
            let Ok(nested) = attr.parse_args_with(parser) else {
                return Ok(false);
            };
            for meta in nested {
                match meta {
                    syn::Meta::List(meta) if meta.path.is_ident("parse_dsl") => {
                        meta.parse_nested_meta(|m| self.parse(m))?;
                        ret = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(ret)
    }
    #[allow(clippy::needless_pass_by_value)] // false positive. Type necessary for calling it
    fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        match () {
            () if meta.path.is_ident("ignore") => {
                *self = Self::Ignore;
                Ok(())
            }
            () => {
                let path = &meta.path;
                let ident = quote!(#path);
                let msg = format!("Unrecognized `parse_dsl` meta attribute: `{ident}`");
                Err(meta.error(msg))
            }
        }
    }
    fn parse_list(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut fn_config = Self::default();
        for attr in attrs.iter().filter(is_parse_dsl_attr) {
            fn_config.parse_attr(attr)?;
        }
        Ok(fn_config)
    }
}

// Similar to [`syn::Path::is_ident`] but checks for [`syn::Type::Path`] and
// may work on generic types where `is_ident` doesn't.
fn is_type(ty: &syn::Type, ident: &str) -> bool {
    if let syn::Type::Path(ty) = ty {
        ty.path.segments.last().is_some_and(|l| l.ident == ident)
    } else {
        false
    }
}
struct TypeParser {
    typ: syn::Ident,
    parser: syn::Expr,
}
impl TypeParser {
    fn is_type(&self, ty: &syn::TypePath) -> bool {
        ty.path.is_ident(&self.typ)
    }
}

pub(crate) struct ImplConfig {
    chirp_crate: syn::Path,
    delegate: Option<syn::Ident>,
    set_params: Option<syn::Generics>,
    type_parsers: Vec<TypeParser>,
}
impl Default for ImplConfig {
    fn default() -> Self {
        Self {
            chirp_crate: syn::parse_quote!(::cuicui_chirp),
            delegate: None,
            set_params: None,
            type_parsers: Vec::new(),
        }
    }
}

impl ImplConfig {
    #[allow(clippy::needless_pass_by_value)] // false positive. Type necessary for calling it
    pub(crate) fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        match () {
            () if meta.path.is_ident("cuicui_chirp_path") => {
                let value = meta.value()?;
                self.chirp_crate = value.parse()?;
            }
            () if meta.path.is_ident("delegate") => {
                let value = meta.value()?;
                self.delegate = Some(value.parse()?);
            }
            () if meta.path.is_ident("set_params") => {
                self.set_params = Some(meta.input.parse()?);
            }
            () if meta.path.is_ident("type_parsers") => {
                meta.parse_nested_meta(|meta| {
                    let Some(ident) = meta.path.get_ident() else {
                        return Err(meta.error("type_parsers type must be an identifier"));
                    };
                    let value = meta.value()?;
                    self.type_parsers
                        .push(TypeParser { typ: ident.clone(), parser: value.parse()? });
                    Ok(())
                })?;
            }
            () => {
                let path = &meta.path;
                let ident = quote!(#path);
                let msg = format!("Unrecognized parse_dsl_impl meta attribute: {ident}");
                return Err(meta.error(msg));
            }
        }
        Ok(())
    }
}

fn parse_dsl_receiver(fn_item: &syn::ImplItemFn) -> Option<()> {
    let receiver = fn_item.sig.receiver()?;
    receiver.reference.as_ref()?;
    receiver.mutability.as_ref()?;
    Some(())
}
fn dsl_function(item: &syn::ImplItem) -> Option<&syn::ImplItemFn> {
    let syn::ImplItem::Fn(fn_item) = item else {
        return None;
    };
    parse_dsl_receiver(fn_item).map(|()| fn_item)
}
fn dsl_function_mut(item: &mut syn::ImplItem) -> Option<&mut syn::ImplItemFn> {
    let syn::ImplItem::Fn(fn_item) = item else {
        return None;
    };
    parse_dsl_receiver(fn_item).map(|()| fn_item)
}

pub(crate) fn parse_dsl_impl(config: &mut ImplConfig, block: &mut syn::ItemImpl) -> TokenStream {
    let this_generics = config.set_params.get_or_insert_with(|| {
        let mut generics = block.generics.clone();
        bind_to_parse_dsl(&config.chirp_crate, &mut generics);
        generics
    });
    let this_type = block.self_ty.as_ref();
    let this_crate = &config.chirp_crate;

    let funs = block.items.iter().filter_map(dsl_function);
    let funs = funs.map(|f| method_branch(f, &config.type_parsers));
    let catchall = config.delegate.as_ref().map_or_else(
        || quote!(Err(DslParseError::<Self>::new(name))),
        |ident| quote!(self.#ident.method(MethodCtx { name, arguments, ctx, registry })),
    );
    let parse_dsl_block = quote! {
        #[automatically_derived]
        #[allow(clippy::let_unit_value)]
        impl #this_generics #this_crate::ParseDsl for #this_type {
            fn method(
                &mut self,
                data: #this_crate::parse_dsl::MethodCtx,
            ) -> Result<(), #this_crate::anyhow::Error> {
                use #this_crate::parse_dsl::{MethodCtx, DslParseError, args};

                let MethodCtx { name, arguments, mut ctx, registry } = data;
                match name {
                    #(#funs)*
                    _name => { #catchall }
                }
            }
        }
    };
    // Remove `parse_dsl` attributes from block items, as otherwise rust
    // vainly tries to understand them.
    for item_fn in block.items.iter_mut().filter_map(dsl_function_mut) {
        item_fn.attrs.retain(|a| !is_parse_dsl_attr(&a));
    }
    quote!(#block #parse_dsl_block)
}

/// Add `: ParseDsl` type bound to `generics`, with given `chirp_crate` as
/// path to `ParseDsl`
fn bind_to_parse_dsl(chirp_crate: &syn::Path, generics: &mut syn::Generics) {
    use syn::TraitBound as Bound;
    use syn::TypeParamBound::Trait;

    for type_param in generics.type_params_mut() {
        let arguments = syn::PathArguments::None;
        let modifier = syn::TraitBoundModifier::None;
        let ident = syn::Ident::new("ParseDsl", chirp_crate.span());

        let mut path = chirp_crate.clone();
        path.segments.push(syn::PathSegment { ident, arguments });
        let bound = Trait(Bound { paren_token: None, lifetimes: None, modifier, path });
        type_param.bounds.push(bound);
    }
}
// Note: assumes cuicui_chirp::parse_dsl::split is in scope and used correctly
fn method_branch(fun: &syn::ImplItemFn, parsers: &[TypeParser]) -> TokenStream {
    match FnConfig::parse_list(&fun.attrs) {
        Ok(FnConfig::Ignore) => return TokenStream::new(),
        Ok(FnConfig::Method) => {}
        Err(err) => {
            // Since we use this as a `pat => match_branch`, we can't simply return
            // the value of err.into_compile_error(). We need to add the pattern,
            // otherwise, we get a syntax error, not the compilation error we want…
            let compile_error = err.into_compile_error();
            return quote!(_ => {#compile_error});
        }
    };
    let arg_parsers = fun.sig.inputs.iter().skip(1);
    let arg_parsers = arg_parsers.map(|a| argument_parser(a, parsers));

    let arg_count = arg_parsers.len();
    let index = syn::Index::from;
    let fun_args = (0..arg_count)
        .map(index)
        .map(|i| quote!(arguments.get_str(#i).unwrap().as_ref()));

    let ident = &fun.sig.ident;

    quote_spanned! { fun.sig.inputs.span() =>
        stringify!(#ident) => {
            if arguments.len() != #arg_count {
                return Err(args::ArgumentError { expected: #arg_count, got: arguments.len() }.into());
            }
            self.#ident(#(#arg_parsers(registry, ctx.as_deref_mut(), #fun_args)?),*);
            Ok(())
        }
    }
}
fn argument_parser(argument: &syn::FnArg, parsers: &[TypeParser]) -> TokenStream {
    use syn::Type::{Path, Reference as Ref};
    use syn::TypeReference as TRef;

    match argument {
        syn::FnArg::Receiver(_) => unreachable!(),
        syn::FnArg::Typed(syn::PatType { ty, .. }) => match ty.as_ref() {
            Path(ty) if parsers.iter().any(|prs| prs.is_type(ty)) => {
                let find = |prs| TypeParser::is_type(prs, ty).then_some(&prs.parser);
                let parser = parsers.iter().find_map(find).unwrap();
                quote!(#parser)
            }
            Path(ty) if ty.path.is_ident("Handle") => quote!(args::to_handle),
            Ref(TRef { elem, .. }) if is_type(elem, "Handle") => quote!(&args::to_handle),
            Ref(TRef { elem, .. }) if is_type(elem, "str") => quote!(&args::quoted),
            _ => quote!(args::from_reflect),
        },
    }
}
