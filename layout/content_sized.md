# `cuicui_layout` Content sized nodes

Nodes may have "content-sized" rules.

When a node has a content-sized rule, A [`ContentSized`] component is added to
it. The layouting algorithm will use the values in the `ContentSized` component
to compute the value of content-sized rules.

For example, in `cuicui_layout_bevy_ui`, a leaf node containing an image may
keep the same aspect ratio as the image.

## How to define content-sized elements?

Just Update the `ContentSized` component with the pixel size value of the
content.

[`ContentSized`]: https://docs.rs/cuicui_layout/0.10.2/cuicui_layout/rule/struct.ContentSized.html
