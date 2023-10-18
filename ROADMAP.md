
- Add chirp utilities (#105, #95)
  - **Breaking**: moved `ReflectDsl` to `cuicui_chirp::utils::reflect`
  - Added `DynamicBundleList`: Add arbitrary `T: Bundle` to it and then consume
    them using `<DynamicBundleList as DslBundle>::insert`.
  - Added `DynamicMarker`: A way to add arbitrary reflected marker components
    based on their name only.
  - Added `TypeRegistryDsl` that accepts as method any type present in the type registry (#95)
- Accept a `&str` whenever we would accept `Handle<T>` in `ReflectDsl` (#78)
- `ParseDsl` has a new provided method: `available_methods`, the list of method
  names implemented by the `ParseDsl`.
- Errors are now much more actionable in `cuicui_chirp` and `cuicui_layout`
  - We provide a "trace" of rule computation when erroring in `cuicui_layout` (#98)
  - We do a second pass with a more tolerent parser when `cuicui_chirp` files
    have invalid syntax, providing more errors at the same time and giving more
    actionable error messages (#103)
  - We use `ParseDsl::available_methods` to give suggestions when encountering
    missing methods
- A morphorm-like `Rule::Stretch`. Interaction with other rules TBD
