use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{meta::ParseNestedMeta, spanned::Spanned};

#[derive(Default, Debug, PartialEq)]
enum FnConfig {
    #[default]
    Method,
    Ignore,
    LeafNode,
}
const METHOD_ATTR_DESCR: &str = "\
- `parse_dsl(ignore)`: Do not add this method to the parse_dsl_impl implementation
- `parse_dsl(leaf_node)`: Add this method to the `ParseDsl::leaf_node` method instead of \
  `ParseDsl::method`. Leaf node methods must follow a specific type signature, \
  please read the following for more details:

https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html#leaf-node

There is currently no other accepted parse_dsl_impl method attribute config options.\n";

#[allow(clippy::trivially_copy_pass_by_ref)] // false positive. Type necessary to avoid eta-expension
fn is_parse_dsl_attr(attr: &&syn::Attribute) -> bool {
    attr.path().is_ident("parse_dsl")
}
impl FnConfig {
    #[allow(clippy::needless_pass_by_value)] // false positive. Type necessary for calling it
    fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        if *self != Self::default() {
            let msg = format!(
                "More than one `parse_dsl` meta attribute was declared \
                for this method, can't know which one to chose! Only use one.\n\
                {METHOD_ATTR_DESCR}"
            );
            return Err(meta.error(msg));
        }
        match () {
            () if meta.path.is_ident("ignore") => {
                *self = Self::Ignore;
                Ok(())
            }
            () if meta.path.is_ident("leaf_node") => {
                *self = Self::LeafNode;
                Ok(())
            }
            () => {
                let msg = format!("Unrecognized `parse_dsl` meta attribute\n{METHOD_ATTR_DESCR}");
                Err(meta.error(msg))
            }
        }
    }
    fn parse_list(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut fn_config = Self::default();
        for attr in attrs.iter().filter(is_parse_dsl_attr) {
            attr.parse_nested_meta(|meta| fn_config.parse(meta))?;
        }
        Ok(fn_config)
    }
}

pub(crate) struct ImplConfig {
    chirp_crate: syn::Path,
    delegate: Option<syn::Ident>,
    set_params: Option<syn::Generics>,
}
impl Default for ImplConfig {
    fn default() -> Self {
        ImplConfig {
            chirp_crate: syn::parse_quote!(::cuicui_chirp),
            delegate: None,
            set_params: None,
        }
    }
}

const CONFIG_ATTR_DESCR: &str = "\
- `cuicui_chirp_path = alternate::path`: specify which path to use for the \
  `cuicui_chirp` crate by default, it is `::cuicui_chirp`
- `delegate = delegate_field`: (optional) Field to delegate `ParseDsl::leaf_node` \
  and `ParseDsl::method` implementations when encountering a name not implemented \
  in this `impl` block. This should be the field you mark with `#[deref_mut]`
- `set_params <C: ParseDsl>`: Instead of re-using the `impl` block's generics \
  in the `impl<XXX> ParseDsl for Type` use the expression within parenthesis.

There is currently no other accepted parse_dsl_impl attribute config options.\n";

impl<'a> ImplConfig {
    #[allow(clippy::needless_pass_by_value)] // false positive. Type necessary for calling it
    pub(crate) fn parse(&mut self, meta: ParseNestedMeta<'a>) -> syn::Result<()> {
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
            () => {
                let msg =
                    format!("Unrecognized parse_dsl_impl meta attribute\n{CONFIG_ATTR_DESCR}");
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
    let syn::ImplItem::Fn(fn_item) = item else { return None; };
    parse_dsl_receiver(fn_item).map(|_| fn_item)
}
fn dsl_function_mut(item: &mut syn::ImplItem) -> Option<&mut syn::ImplItemFn> {
    let syn::ImplItem::Fn(fn_item) = item else { return None; };
    parse_dsl_receiver(fn_item).map(|_| fn_item)
}

pub(crate) fn parse_dsl_impl(config: &ImplConfig, block: &mut syn::ItemImpl) -> TokenStream {
    let this_generics = config.set_params.as_ref().unwrap_or(&block.generics);
    let this_type = block.self_ty.as_ref();
    let this_crate = &config.chirp_crate;

    let functions = || block.items.iter().filter_map(dsl_function);
    let leaf_node_fn = leaf_node_fn_impl(config, functions);
    let method_fn = method_fn_impl(config, functions);
    let parse_dsl_block = quote! {
        #[automatically_derived]
        #[allow(all)]
        impl #this_generics #this_crate::ParseDsl for #this_type {
            #method_fn
            #leaf_node_fn
        }
    };
    // Remove `parse_dsl` attributes from block items, as otherwise rust
    // vainly tries to understand them.
    for item_fn in block.items.iter_mut().filter_map(dsl_function_mut) {
        item_fn.attrs.retain(|a| !a.path().is_ident("parse_dsl"));
    }
    quote!(#block #parse_dsl_block)
}
fn method_fn_impl<'a, I: Iterator<Item = &'a syn::ImplItemFn>>(
    config: &ImplConfig,
    functions: impl FnOnce() -> I,
) -> TokenStream {
    let this_crate = &config.chirp_crate;
    let funs = functions().map(method_branch);
    let catchall = config.delegate.as_ref().map_or_else(
        || quote!(Err(DslParseError::<Self>::new(name, ParseType::Method))),
        |ident| quote!(self.#ident.method(InterpretMethodCtx { name, args })),
    );
    quote! {
        fn method(
            &mut self,
            data: #this_crate::parse::InterpretMethodCtx,
        ) -> Result<(), #this_crate::anyhow::Error> {
            use #this_crate::parse::{quick, InterpretMethodCtx, DslParseError, ParseType};
            let InterpretMethodCtx { name, args } = data;
            match name {
                #(#funs)*
                name => { #catchall }
            }
        }
    }
}
fn leaf_node_fn_impl<'a, I: Iterator<Item = &'a syn::ImplItemFn>>(
    config: &ImplConfig,
    functions: impl FnOnce() -> I,
) -> TokenStream {
    let this_crate = &config.chirp_crate;
    let funs = functions().map(leaf_node_branch);
    let catchall = config.delegate.as_ref().map_or_else(
        || quote!(Err(DslParseError::<Self>::new(name, ParseType::LeafNode))),
        |ident| quote!(self.#ident.leaf_node(InterpretLeafCtx { name, leaf_arg, cmds })),
    );
    quote! {
        fn leaf_node(
            &mut self,
            data: #this_crate::parse::InterpretLeafCtx,
        ) -> Result<#this_crate::bevy_types::Entity, #this_crate::anyhow::Error> {
            use #this_crate::parse::{quick, InterpretLeafCtx, DslParseError, ParseType};
            let InterpretLeafCtx { name, leaf_arg, mut cmds } = data;
            match name {
                #(#funs)*
                name => { #catchall }
            }
        }
    }
}
// Note: assumes cuicui_chirp::parse::quick is in scope and used correctly
fn method_branch(fun: &syn::ImplItemFn) -> TokenStream {
    match FnConfig::parse_list(&fun.attrs) {
        Ok(FnConfig::LeafNode | FnConfig::Ignore) => return TokenStream::new(),
        Err(err) => return err.into_compile_error(),
        _ => {}
    };
    let index = syn::Index::from;
    let arg_count = fun.sig.inputs.len() - 1;
    let arg_n = format_ident!("arg{arg_count}", span = fun.sig.inputs.span());
    let fun_args = (0..arg_count).map(index).map(|i| quote!(args.#i));
    let ident = &fun.sig.ident;
    quote_spanned! { fun.sig.inputs.span() =>
        stringify!(#ident) => {
            let args = quick::#arg_n(args)?;
            self.#ident(#(#fun_args),*);
            Ok(())
        }
    }
}
fn leaf_node_branch(fun: &syn::ImplItemFn) -> TokenStream {
    match FnConfig::parse_list(&fun.attrs) {
        Ok(FnConfig::Method | FnConfig::Ignore) => return TokenStream::new(),
        Err(err) => return err.into_compile_error(),
        _ => {}
    };
    let ident = &fun.sig.ident;
    quote_spanned! { fun.sig.inputs[1].span() =>
        stringify!(#ident) => {
            let arg = quick::arg1(leaf_arg)?;
            Ok(self.#ident(arg, &mut cmds))
        }
    }
}
