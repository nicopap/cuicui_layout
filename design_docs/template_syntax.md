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

### Decorating the "root" in the call site

Problem with this approach: We'd like to name the root entity & call more
methods on it.

Options:

1. Call the template as a method
2. Special syntax for methods in template calls

```ron
// (1)
Menu(column rules(420px, 100%) main_margin(40) image("images/main_menu/board.png")) {
    LoadGame(main_menu_item!("LOAD GAME"))
    Settings(main_menu_item!("SETTINGS") swatch_target(Settings))
    QuitGame(cancel main_menu_item!("QUIT GAME"))
    EmptyBottomMargin(rules(95%, 10%))
}
// (2)
Menu(column rules(420px, 100%) main_margin(40) image("images/main_menu/board.png")) {
    main_menu_item!("LOAD GAME")
    main_menu_item!("SETTINGS" | swatch_target(Settings))
    main_menu_item!("QUIT GAME" | cancel)
    EmptyBottomMargin(rules(95%, 10%))
}
// (3)
main_menu_item!("QUIT GAME")(cancel)
// (4)
main_menu_item!("QUIT GAME"){cancel}
```

Problem with (1) is that we break the concept of statement. The template is
suppose to spawn a single entity.

Problem with (2) is that some DSLs could overwrite previous values, and adds
new syntax. With (2) we also can't name the entity at the call site.

(3) is silly (4) breaks the assumption that things between `{}` are statements.

### Multiple parameters calls

Other problem: When template is placed in statement position, the thing between
parenthesis looks like a method site.

But in the template definition, we used commas to separate arguments! So there
is a contradiction between expectations at call site and expectations from
previous understanding of languages.

Also, we'd like to be able to substitute individual names with several token
(TokenTree rule), so splitting on comma would be good.

Also, with the `|` syntax, it becomes a bit strange that two sides of the pipe
inside parenthesis has different ways of splitting arguments (commas on the
left and spaces on the right)

**Decision**: while completely silly, the `()()` syntax makes more sense than
`|`. I want to use commas in the templates, while using spaces in method site.
It also makes sense with the ability to add `{}` afterward.

## Relationship with the `dsl!` macro

We can't use a similar mechanism in the `dsl!` macro. The rust module system
is independent from the chirp module system.

However, we already used something pretty similar in `chirpunk`. We have a
`element: DslElement` field on the `ChirpunkDsl`, `element` is set based on a
method. For example, method named `tab_button` sets it to `element = DslElement::TabButton`.
Then, in `DslBundle::insert` impl, we call the `element.spawn` method on `DslElement`.
`spawn` calls a function that is more or less the same as the `fn` templates we
are describing in this page.