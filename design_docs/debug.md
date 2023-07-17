# Debug overlay

## Features

- Display node info at top left of container
  - If node has `Name`, then show it. Use entity debug value if not
  - `PosRect.size`
  - for `Container`: flow, distribution & alignment as a `FdDaA` string
     - `Flow::Vertical`: `v`
     - `Flow::Horizontal`: `>`
     - `Distribution::Start`, `Alignment::Start`: `S`
     - `Distribution::End`, `Alignment::End`: `E`
     - `Distribution::FillMain`, `Alignment::Center`: `C`
  - Tooltip hover for even more details!
    - `PosRect.pos`
    - margin
    - rules
    - fully-named distrib/align/flow
    - explanation of the short string
    - explanaiton of rule arrows
  - Text MUST NOT overlap with other node's info
    - For smaller containers, replace text with hoverable icon
    - For containers dispalce text bellow existing text.
    - If the displacement is more than 50% of the container's height
      bellow the container it is supposed to describe, it should be elided.
    - Try to avoid overlaping with rule arrows (maybe by
      placing the arrows after text, so that it doesn't overlap)
  - May need to give a semi-transparent darker background for
    readability
- Distinct (quasi-random sequence) color for each node
  - The text should have the same color as the one chosen for the
    node
  - Container size is outlined (inset) with a gizmo box
  - margins are highlight with a color of same hue, but higher
    luminance
  - different containers with exact same position should still be
    visible by insetting even more the inner container
- Visualize rules as follow:
  - An arrow centered on the box, with a size equals to the smallest
    of either 100px or 25% of container's size
  - Arrow points outward on `Rule::Parent` axis, with the percentage
    written on top for width-rules and at the right for height-rules
  - Arrow points inward on `Rule::Children` axis, with ratio
    displayed next to it.
  - Arrow points inward on `Rule::Fixed` content-dependent, no text.
  - No arrows on fixed-size axis.
- Toggle between different display modes:
  - No debug overlay
  - Only outlines (including margins)
  - Outlines + rules
  - Outlines + rules + tooltips (shift only)
  - Outlines + rules + tooltips + text
- Highlight nodes that causes a layouting error, and it's largest child
  - Use RED or a larger outline.

```text
The spec string in the form:

`vdSaS` or `FdDaA`

tells the containers' properties:

- `F in [v>]`: The `Flow`, `v = Vertical; > = Horizontal`
- `D in [SEC]`: The `Distribution`, `S = Start; E = End; C = FillMain`
- `A in [SEC]`: The `Alignment`, `S = Start; E = End; C = Center`

so that `vdSaS` has the following properties: `Flow::Vertical`, `Distribution::Start` and `Alignment::Start`.
```
