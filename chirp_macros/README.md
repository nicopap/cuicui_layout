# cuicui format `ParseDslImpl` macro

Generate `ParseDsl` implementations to parse chirp files from an `impl` block.

## Behavior

```text
use cuicui_chirp::{ParseDslImpl, ParseDsl};

#[parse_dsl_impl(set_params <D: ParseDsl>, delegate = inner)]
impl<D: DslBundle> LayoutDsl<D> {
    #[parse_dsl(ignore)]
    pub fn flow(&mut self, flow: Flow) {
    }
    pub fn column(&mut self) {
    }
    pub fn row(&mut self) {
    }
    pub fn rules(&mut self, width: Rule, height: Rule) {
    }

    // ...

    #[parse_dsl(leaf_node)]
    pub fn empty_px(&mut self, pixels: u16, cmds: &mut EntityCommands) -> Entity {
      todo!()
    }
    #[parse_dsl(leaf_node)]
    pub fn spawn_ui<M>(
        &mut self,
        ui_bundle: impl IntoUiBundle<M>,
        cmds: &mut EntityCommands,
    ) -> Entity {
      todo!()
    }
}
```

All methods with a `&mut self` argument
will automatically be added to the `ParseDsl::method` implementation.

To add a method to the `ParseDsl::leaf_node` implementation instead, use the
`#[parse_dsl(leaf_node)]` attribute. To ignore completely a function in the
impl block, use `#[parse_dsl(ignore)]`.

This relies on the `FromStr` trait, each non-self argument to a `method` or
`leaf_node` should implement `FromStr`, so that it is possible to parse it
from a string.

If you dont properly mark [leaf nodes], you'll get a compilation error, as
`&mut EntityCommands` does not implement `FromStr`.

For the snippet of code shown earlier, the macro output will be:

1. The block as-is, without modifications
2. A `impl ParseDsl for LayoutDsl` as follow:

```text
impl<D: ParseDsl> ParseDsl for LayoutDsl<D> {
    fn method(
        &mut self,
        data: cuicui_chirp::parse::InterpretMethodCtx,
    ) -> Result<(), cuicui_chirp::anyhow::Error> {
        use cuicui_chirp::parse::{quick, InterpretMethodCtx, DslParseError, ParseType};
        let InterpretMethodCtx { name, args } = data;
        match name {
            stringify!(column) => {
                let args = quick::arg_0(args)?;
                self.column();
                Ok(())
            }
            stringify!(row) => {
                let args = quick::arg_0(args)?;
                self.row();
                Ok(())
            }
            stringify!(rules) => {
                let args = quick::arg_2(args)?;
                self.rules(args.0, args.1);
                Ok(())
            }
            name => {
                self.inner.method(InterpretMethodCtx { name, args })
            }
        }
    }
    fn leaf_node(
        &mut self,
        mut data: cuicui_chirp::parse::InterpretLeafCtx,
    ) -> Result<Entity, anyhow::Error> {
        use cuicui_chirp::parse::{quick, InterpretLeafCtx, DslParseError, ParseType};
        let InterpretLeafCtx { name, leaf_arg, cmds } = &mut data;
        match name {
            stringify!(empty_px) => {
                let arg = quick::arg1(leaf_arg)?;
                Ok(self.button(arg, cmds))
            }
            stringify!(spawn_ui) => {
                let arg = quick::arg1(leaf_arg)?;
                Ok(self.spawn_ui(arg, cmds))
            }
            name => {
                self.inner.leaf_node(InterpretLeafCtx { name, leaf_arg, cmds })
            }
        }
    }
}
```

[leaf nodes]: https://docs.rs/cuicui_dsl/latest/cuicui_dsl/macro.dsl.html#leaf-node
