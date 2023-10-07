# Define your own custom DSL

[![The Book](https://img.shields.io/badge/The_Cuicui_Book-blue)](https://cuicui.nicopap.ch/introduction.html)

[`cuicui_dsl`] and [`cuicui_chirp`] are parametrized over the [`DslBundle`] and
[`ParseDsl`] traits respectively.

You can directly use one of the DSLs exported by an external crate such as
[`UiDsl`], [`LayoutDsl`], [`NavigationDsl`] or [`SpriteDsl`], but we recommend
that you define your own DSL on top of them.

This is how [the chirpunk example] works. We re-use pre-existing DSLs, but add
our own layer on top, to create a unique vocabulary that applies to the specifc
game we build.

[`cuicui_dsl`] and [`cuicui_chirp`] make creating new DSLs the most trivial.
