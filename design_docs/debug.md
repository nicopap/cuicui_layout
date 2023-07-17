# Debug overlay

## Features

- [ ] Display node info at top left of container
  - [ ] If node has `Name`, then show it. Use entity debug value if not
  - [ ] `PosRect.size`
  - [ ] for `Container`: flow, distribution & alignment as a `FdDaA` string
     - [ ] `Flow::Vertical`: `v`
     - [ ] `Flow::Horizontal`: `>`
     - [ ] `Distribution::Start`, `Alignment::Start`: `S`
     - [ ] `Distribution::End`, `Alignment::End`: `E`
     - [ ] `Distribution::FillMain`, `Alignment::Center`: `C`
  - [ ] Tooltip hover for even more details!
    - [ ] `PosRect.pos`
    - [ ] margin
    - [ ] rules
    - [ ] fully-named distrib/align/flow
    - [ ] explanation of the short string
    - [ ] explanaiton of rule arrows
  - [ ] Text MUST NOT overlap with other node's info
    - [ ] For smaller containers, replace text with hoverable icon
    - [ ] For containers dispalce text bellow existing text.
    - [ ] If the displacement is more than 50% of the container's height
      bellow the container it is supposed to describe, it should be elided.
    - [ ] Try to avoid overlaping with rule arrows (maybe by
      placing the arrows after text, so that it doesn't overlap)
  - [ ] May need to give a semi-transparent darker background for
    readability
- [X] Distinct (quasi-random sequence) color for each node
  - [ ] The text should have the same color as the one chosen for the
    node
  - [X] Container size is outlined (inset) with a gizmo box
  - [X] margins are highlight with a color of same hue, but higher
    luminance
  - [X] different containers with exact same position should still be
    visible by insetting even more the inner container
- [X] Visualize rules as follow:
  - [X] An arrow centered on the box, with a size equals to the smallest
    of either 100px or 25% of container's size
  - [X] Arrow points outward on `Rule::Parent` axis
  - [X] Arrow points inward on `Rule::Children` axis
  - [X] Arrow points inward on `Rule::Fixed` content-dependent, no text.
  - [X] No arrows on fixed-size axis.
  - [ ] relevant percentages displayed on top/right-of arrows
- [ ] Toggle between different display modes:
  - [X] No debug overlay
  - [X] Only outlines (including margins)
  - [X] Outlines + rules
  - [ ] Outlines + rules + tooltips (shift only)
  - [ ] Outlines + rules + tooltips + text
- [ ] Highlight nodes that causes a layouting error, and it's largest child
  - [ ] Use RED or a larger outline.

```text
The spec string in the form:

`vdSaS` or `FdDaA`

tells the containers' properties:

- `F in [v>]`: The `Flow`, `v = Vertical; > = Horizontal`
- `D in [SEC]`: The `Distribution`, `S = Start; E = End; C = FillMain`
- `A in [SEC]`: The `Alignment`, `S = Start; E = End; C = Center`

so that `vdSaS` has the following properties: `Flow::Vertical`, `Distribution::Start` and `Alignment::Start`.
```
