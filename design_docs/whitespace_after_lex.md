# How to handle whitespaces with lexer?

**Problem**: `.recognize()` includes whitespace (and comments) between last
token and one parsed. How to not do that?

* If I consume whitespaces _after_ produced token, then the whitespaces are
  at the end, which is as problematic.
* I could offset by computed whitespaces in `.next_slice()`
* Not that costly, only thing that uses SWAR.
* Actually it consumes the next token. Too bad, it seems complicated to extract
  the relevant logic, and most of the time, next token is cheap.

## Alternative

- Add a "pre-whitespace" field to `Input`, use this instead of `advanced.start`
  in `offset_from`
  - Requires consuming whitespaces _after_ produced tokens.
- Split lexer in two:
  1. whitespaces/comments
  2. tokens/ident/quotes

## Other issues

This is also a problem for `with_span()` which is admitedly much more important
to get right.