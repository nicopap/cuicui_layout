# Chirp interpreter error reporting

Will be based off of `miette`. Ideally, there is some form of error recovery
so that we can log further errors.

## File identification

We should use the `LoadContext::path` return value (if `Some`) and fallback to
just using the full source.

(supposedly that's what the `source_code` attribute does in the miette README)

## Spans

How do I get spans. Let's see the `Located` docs (not helpful).

`Parser::span` and `Parser::with_span` is a bit better.