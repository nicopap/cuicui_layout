# Loading images in chirp

It's not enough to use `LoadContext::get_handle`, you need to load the associated
path as well. Which complicates things greatly.

Since I can't predict the sensible way of reading the file type, it seems
perilous.

I'll just have a trait for loading from file :(

## Bevy 0.12 Assets v2

We now can use assets v2 to do this, awesome.
