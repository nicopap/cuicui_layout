# The best color separation

We want a sequence where each subsequent value is maximally separate in [0..1], cylcing.

I initially came up with my own scheme, but after looking up oeis, I found
that I reinvented the Van Der Corput sequence. Do I feel silly.

## Denominator

this is the sequence: https://oeis.org/A062383

```
a(0) = 1
a(n) = 2^⌊log₂(n)+1⌋
a(n) = 2 × a(⌊n/2⌋)
a(n) = n.next_power_of_two()
```

The one problem is that we need to keep track of `n`. I suppose a global
shouldn't be too much of an issue.

Maybe we could "hash" (ie: use raw bit pattern) of then `Entity`

## Nominator

https://oeis.org/A030101

```scala
n => Integer.parseInt(Integer.toString(n, 2).reverse, 2)
```

Probably worth skipping the conversion into a string. Especially given numbers
are already represented as binary on the computer.

```rust
let leading_zeros = n.leading_zeros();
n.reverse_bits() >> leading_zeros
```


## Summing it up

```rust
fn color(entity: Entity) -> Color {
  let bits = entity.to_bits();

  let leading_zeros = if bits == 0 { 0 } else { n.leading_zeros() };
  let nominator = n.reverse_bits() >> leading_zeros;
  let denominator = bits.next_power_of_two();

  let hue = nominator as f32 / denominator as f32;
  // Don't forget to multiply by 360
}
```

See a summary with videos at: https://github.com/bevyengine/bevy/pull/9175