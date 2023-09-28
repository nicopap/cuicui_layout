# Template implementation

## Do this in multiple steps

### 1. Templates without arguments

Store the list of templates somewhere, check them when encountering a `foo!`.

### 2. Templates with additional methods & children

We need to make sure the `template!()(method) {Children}` syntax work

### 3. Templates with single argument

- **Problem 1**: call graph and keeping track of scope.
- **Problem 2**: inlining the arguments when parsing.
- **Problem 3**: Keeping track of correct offset.

Idea (2): Argument positions: only when calling other templates or in method
arguments. We should provide a `Cow<[u8]>` to argument methods. (We already
use a `Cow<str>` when converting to string, so it should be fine for this as
well)

But maybe we want arguments to be able to expand to whole method calls?

### 4. Templates with many arguments

**Problem**: Splitting arguments and chosing correct one.

### 5. Template imports
