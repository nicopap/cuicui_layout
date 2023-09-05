# Avoid panics in `Dsl` Methods

**Problem**: Panics in DSL methods are fatal.

We may depend on the DSL being fully parsed to have a coherent state to read
and spawn.

Our interpreter is resilient, it just accumulates errors and runs the next method.

Two previous paragraphs are in contradiction.

## Solution

1. Tell users to not panic in methods
2. Do not call `insert` or `node` when error list is non-empty in interpreter
