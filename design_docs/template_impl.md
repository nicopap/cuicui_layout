# Template implementation

## Do this in multiple steps

### 1. Templates without arguments

Store the list of templates somewhere, check them when encountering a `foo!`.

### 2. Templates with additional methods & children

We need to make sure the `template!()(method) {Children}` syntax work

**Problem**: We can't simply apply the methods after the template call, because
we already spawned the template's body entity. And those two things are different:

```rust
// all methods on single DSL
let mut dsl = Dsl::default(); dsl.x(); dsl.y(); dsl.insert(cmds);
// two instances of DSL
let mut dsl = Dsl::default(); dsl.x(); dsl.insert(cmds);
let mut dsl = Dsl::default(); dsl.y(); dsl.insert(cmds);
```

So we need to pass the methods to the template, so that they can be applied, and
ideally, at the end of the body entity's methods list.

### 3. Templates with single argument

- **Problem 1**: call graph and keeping track of scope.
- **Problem 2**: inlining the arguments when parsing.
- **Problem 3**: Keeping track of correct offset.

Idea (2): Argument positions: only when calling other templates or in method
arguments. We should provide a `Cow<[u8]>` to argument methods. (We already
use a `Cow<str>` when converting to string, so it should be fine for this as
well)

But maybe we want arguments to be able to expand to whole method calls?

- **Problem 4**: "Forwarding" arguments to template calls

When within a template, we want to pass along values from arguments. But how?

Well an idea is to the same as for the template extras, which is to walk back
the call stack and check for presence of variable in past scopes.

But this doesn't work because we also need to know from which scope the variable
comes

But it works, because it can only come from the scope directly above.

#### In short: global parameter substitution algorithm

This includes managing template extras.

For any given scope.
When walking the statement tree
for the root statement:
- If the root statement is a template call
  - Add the template extras
  - Enter new scope where the **arguments** used to call the template are associated
    with the **parameters** of the template being called
- call methods, substituing

### 4. Templates with many arguments

**Problem**: Splitting arguments and chosing correct one.

### 5. Template imports
