# Template call syntax

The proposed syntax was as follow (<https://github.com/nicopap/cuicui_layout/issues/80>):

```ron
use template_name(100%, coral)
```

I do not like this syntax. When it's embedded in a chirp file it looks as follow:


```ron
Header(row width(100%) height(10%) main_margin(50)) {
    Entity(column rules(100%, 100%)) {
        TabButtons(row rules(100%, 99%)) {
            RETURN(cancel text("RETURN") width(19.9%) style(BackText))
            TabBar(row width(79.9%) main_margin(100)) {
                use tab_button("DISPLAY", Display)
                use tab_button("SOUND", Sound)
                use tab_button("TAB 3", Tab3)
                use tab_button("TAB 4", Tab4)
            }
        }
        use tab_line_separator.wgsl
    }
}
```

This just looks weird, and the rust metaphore is completely lost. I suggest using
a syntax similar to the rust macro system. After all, templates are a form of
macros:

```ron
Header(row width(100%) height(10%) main_margin(50)) {
    Entity(column rules(100%, 100%)) {
        TabButtons(row rules(100%, 99%)) {
            RETURN(cancel text("RETURN") width(19.9%) style(BackText))
            TabBar(row width(79.9%) main_margin(100)) {
                tab_button!("DISPLAY", Display)
                tab_button!("SOUND", Sound)
                tab_button!("TAB 3", Tab3)
                tab_button!("TAB 4", Tab4)
            }
        }
        tab_line_separator!()
    }
}
```

Issue is:

- File-based templates (ie: no arguments) are difficult to associate with their
  actual source

This can be improved by using an actual `use` statement.

```ron
// alternatives:
use tab_line_separator
use "tab_line_separator.wgsl"
use tab_line_separator.wgsl as tab_line_separator
// for specific macro imports (ones defined with `fn`)
use widgets/tab_button
use widgets/cancel_button

Header(row width(100%) height(10%) main_margin(50)) {
    Entity(column rules(100%, 100%)) {
        TabButtons(row rules(100%, 99%)) {
            RETURN(cancel text("RETURN") width(19.9%) style(BackText))
            TabBar(row width(79.9%) main_margin(100)) {
                tab_button!("DISPLAY", Display)
                tab_button!("SOUND", Sound)
                tab_button!("TAB 3", Tab3)
                tab_button!("TAB 4", Tab4)
            }
        }
        tab_line_separator!()
    }
}
```

Ideally, we also have a privacy system that allows:

- Declaring whole files as embeddables
- Declaring individual functions as useable in other chirp files.

## Relationship with the `dsl!` macro

We can't use a similar mechanism in the `dsl!` macro. The rust module system
is independent from the chirp module system.

However, we already used something pretty similar in `chirpunk`. We have a
`element: DslElement` field on the `ChirpunkDsl`, `element` is set based on a
method. For example, method named `tab_button` sets it to `element = DslElement::TabButton`.
Then, in `DslBundle::insert` impl, we call the `element.spawn` method on `DslElement`.
`spawn` calls a function that is more or less the same as the `fn` templates we
are describing in this page.