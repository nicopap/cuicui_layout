# Chirp `ParseDsl` impl block macro

Generate `ParseDsl` implementations to parse chirp files from an `impl` block.

## Behavior

```rust,ignore
use cuicui_chirp::{parse_dsl_impl, ParseDsl};

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

    #[parse_dsl(ignore)]
    pub fn empty_px(&mut self, pixels: u16, cmds: &mut EntityCommands) -> Entity {
      todo!()
    }
}
```

All methods with a `&mut self` argument
will automatically be added to the `ParseDsl::method` implementation.

To ignore completely a function in the impl block, use `#[parse_dsl(ignore)]`.

For the snippet of code shown earlier, the macro output will be:

1. The block as-is, without modifications
2. A `impl ParseDsl for LayoutDsl` as follow:

```rust,ignore
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
}
```
