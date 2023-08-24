# `cuicui` crates formatting recommendation

This only apply to top level items.

1. Avoid single `use` that takes more than a line
2. Declare `use`s before `mod`s.
3. Split `use`s between `std`, extern crates and local crate imports (may change in the future)
4. `use`, `pub use`, `mod`, `pub mod`