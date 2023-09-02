# Parsing method arguments in a "clean" way

**Aim 1**: reproduce rust syntax.

Typical thing that should work:

```rust
method(10, 34.3)
method(Enum::Value);
method("some string", "some other string", Enum::Value, 34.3);
method("some string");
```

**Aim 2**: Concise, not too noisy. I'm considering a special syntax for
single argument methods that accept a string:

```rust
method(some string);
```

This is mostly to make specifying windows paths a bit less insane:

```rust
method("some\\path\\to\\scene.chirp");
```

But not too hot on it since it would confuse syntax highlighters. I think it
would make more sense with the proposed `=` single-binding syntax.

## Parsing strings

One thing to note is that we want to:

1. Preserve the string as-is when passing to the `wrapargs` parser, so that
   the reflection-based parser can handle them nicely.
2. Interpret the string when it is a top level element (ie: method accepts `&str`)
3. Account for the string syntax in the parser, so that we can split properly
   on commas.
