# Import system

- You can mark templates as publicly exposed by prefixing `fn` with `pub`.
- If there is any `pub` template in the file, then there cannot be a root statement
- You can import templates using the `use` statement.

## Exporting

The `Chirp` asset loaded by the asset loader is an enum with `Export`, `Scene`
variants.

## Importing

Interpreting is now split in two:

1. Import stage: it returns the list of imported files
2. Interpret stage: Given the imported files, interpret the rest of the file,
   returning the `Export` or `Scene`

This allows the asset loader to asynchronously load the imported assets, while
not having to care about async in the interpreter (which would be absolute
hell to handle)