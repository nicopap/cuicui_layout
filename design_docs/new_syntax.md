## Grammar

```ungrammar
TokenTree
   = 'ident'
   | '(' (TokenTree)* ')'
   | '[' (TokenTree)* ']'
   | '{' (TokenTree)* '}'
   | StringLit

Method = 'ident' ('(' (TokenTree)* ')')?

Statement
   = 'code'    '(' 'ident' ')'
   | 'Entity'  StatementTail
   | 'ident'   StatementTail
   | StringLit StatementTail

StatementTail
   = '(' (Method)* ')' ('{' (Statement)* '}')?
   | '{' (Statement)* '}'
```

* Notice how `StatementTail` is **never empty**. This ensures that syntax errors
  such as `My Entity Name()` are detected and reported correctly.
* `'ident'` is any series of character that is not a whitespace or delimiter such
  as `=[]{}()"'`, so this includes surprising stuff such as `+-_` and `234`.
* `StringLit` works similarly to a rust string literal.
* `TokenTree` aims to work like a [rust `TokenTree`], may accept more than what
  the rust grammar accepts (within acceptable limits) for performance reason.
  The inside of the parenthesis is passed as-is to `ParseDsl::method`.

#### Comments

`//` are token of themselves. An identifier can contain a `//`, so line comments
must have at least one space after an identifier.

#### Note on `parse_dsl_impl`-specific details

* `parse_dsl_impl` splits the `TokenTree`s on comma, and passes the split arguments
  to the methods defined in the `impl` blocks it decorates.
* It checks how many arguments there are when split on comma, to raise an error
  when too many or too little arguments are passed to the method.
* If an individual argument `TokenTree` is a string literal, it also removes the
  quotes and applies backslash escapes before passing it to the method.

[rust `TokenTree`]: https://doc.rust-lang.org/reference/macros.html#macro-invocation
