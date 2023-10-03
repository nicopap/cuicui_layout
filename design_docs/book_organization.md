# How to organize the book

The idea is to make each page:

- An introduction that would be the repository README, and eventually the `cuicui` README.
  - Should contain at least one snippet of code
- the `README.md` of the corresponding crate, (`dsl`, `chirp` and `layout`)
- Additional details for `layout::debug`, which could also be imported as individual docs
- Additional details for `layout::content_sized`, which could also be imported as individual docs
- Not inlcude `ui` and `sprite`, but instead write an outline (and links to docs)

This would allow testing each pages examples through rust's doc example runner.
