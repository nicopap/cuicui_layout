# Reactivity

> This chapter is **WIP**, the factual accuracy, pedagogic strength, and code weren't tested.

An extremely serious scientific study[^1] that surveyed many developpers revealed
that UI is most often used to display data and accept input from users.

Up to now, all we did was draw some text and ugly pictures on the screen.

Time to juice up our menu.

## Juicing up

**But first, a disclaimer**: The `cuicui` framework, by nature, is extremely
_unopinionated_ when it comes to how interaction works. Currently, `cuicui`
has in fact no built-in way of dealing with data that evolves over time
(such as game objects), or reacting to user input. `cuicui` focuses on scene
definition and layouting.

We will be using 3rd party crates to add reactivity. We will use [`bevy_framepace`]
and [`bevy_mod_picking`] for this example:

```toml
# Enable "frame pacing", a way to reduce input latency
bevy_framepace = "0.13.3"

# Add mouse interaction
bevy_mod_picking = { version = "0.15.0", default-features = false, features = [
    "backend_bevy_ui",
] }
```

## Reacting to click events

The `bevy_mod_picking` docs are a bit sparse. But the basics is to add a [`On`]
component to the relevant entities with a closure to run as callback.

Issue is that `On::run` accepts a callback, and it's impossible to define a
callback in a chirp file. So what do we do?

The answer is to define the callbacks in rust files. Let's add some methods to
our custom DSL.

```rust,no_run,noplayground
enum UiAction {
    PrintHello,
    PrintGoodbye,
}
struct BetterFactorioDsl {
    inner: UiDsl,
    action: UiAction,
}
#[parse_dsl_impl(delegate = inner)]
impl BetterFactorioDsl {
    // ...

    fn print_hello(&mut self) {
        self.action = UiAction::PrintHello;
    }

    fn print_goodbye(&mut self) {
        self.action = UiAction::PrintGoodbye;
    }

}
impl DslBundle for BetterFactorioDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        type OnClick = On<Pointer<Click>>;

        match self.action {
            UiAction::PrintHello =>
                cmds.insert(OnClick::run(|| info!("Hello world!"))),

            UiAction::PrintGoodbye =>
                cmds.insert(OnClick::run(|| info!("Farewell, odious world!"))),
        };
        cmds.id()
    }
}
```

Now we can use `print_hello` and `print_goodbye` as methods in our chirp files!

```rust,no_run,noplayground
// Show the chirp file with print_hello
```

Now, clicking on any button will print "Hello world!" or "Farewell, odious world!"
into the console.

In fact you could really do anything in those `bevy_mod_picking` callbacks.
Because they are nothing less than whole systems!
You can add a `Query` or `Commands` to the callback parameters to affect the bevy world.

### Using method arguments

Suppose you have _many_ possible actions. with this approach, you would need to
define an individual method for _each_ action, kinda blows eh?

A solution is to accept `UiAction` as argument to the method. `cuicui_chirp`
will use the `UiAction` reflect impl to convert the text representation into
a rust type.

```rust,no_run,noplayground
#[derive(Reflect)]
enum Item {
    ConveyorBelt,
    Assembler,
    Chest,
}
// (1) derive Reflect
#[derive(Reflect)]
enum UiAction {
    PrintHello,
    PrintGoodbye,
    Craft(Item),
    Abdicate,
    InvadeCountry,
    LaunchMissils,
    // a lot of possible actions
}
impl BetterFactorioDsl {
    // ...

    // (2) Accept a UiAction
    fn action(&mut self, action: UiAction) {
        self.action = action;
    }
}
```

```rust,no_run,noplayground
GameMenu {
    DiplomacyMenu {
        button!(InvadeCountry)
        button!(LaunchMissils)
        button!(Abdicate)
    }
    CraftMenu {
        button!(Craft(ConveyorBelt))
        button!(Craft(Assembler))
        button!(Craft(Chest))
    }
    DebugMenu {
        button!(PrintHello)
        button!(PrintGoodbye)
    }
}
```

That's it!

## Evolving data

There is _currently_ no nice way to hook up text (or anything) to data in the
ECS. [I've tried to design something][cuicui_richtext], but it's still a massive
WIP.

So here is how you would change text based on game state in current-day bevy.

1. Add a component to identify entities which text to change based on other
   entities
2. Write a system that reads from the "watched" entity and updates the text of
   the text entity.

```rust,no_run,noplayground
#[derive(Component)]
struct ShowPlayerStat;

#[derive(Component)]
struct ShowAssemblersThroughput;

fn show_player_stat(
    mut texts: Query<&mut Text, With<ShowPlayerStat>>,
    player: Query<
        (&Health, &Int),
        (With<Player>, Or<(Changed<Health>, Changed<Int>, Changed<Player>)>),
    >,
) {
    let Ok((health, int)) = player.get_single() else {
        return;
    };
    for mut text in &mut texts {
        text.sections[HEALTH_SECTION].value = health.to_string();
        text.sections[INT_SECTION].value = int.to_string();
    }
}

// Add this system with a run condition to avoid running it each frame
// `show_assemblers_throughput.run_if(any_changed::<ThroughPut>()))`
fn show_assemblers_throughput(
    mut texts: Query<&mut Text, With<ShowAssemblersThroughput>>,
    assemblers: Query<&Throughput, With<Assembler>>,
) {
    let throughput: f32 = assemblers.iter().map(f32::from).sum();
    for mut text in &mut texts {
        text.sections[THROUGHPUT_SECTION].value = throughput.to_string();
    }
}
```

It's fairly verbose, but currently it's the best approach in bevy.

### List of items

So in our _Better Factorio_ game, we can _craft_ things. Let's add an inventory
to see which items we crafted.

This also reveals a limitation[^2] in `cuicui_layout`. We want an infinite
scrollable list of items, but remember, **it is an error for children to overflow
their parent**!

A property of `cuicui_layout` we will take advantage of to make our infinite
list is that layout trees are self-contained. We will set the scroll area as
a leaf node and add the inventory list as a `Root` container spawned as child
of the leaf node (eheh…). You follow? We basically "cut the link" between
the parent and child node, so that the layout algorithm applies independently
to the inventory window and its content.


```rust,no_run,noplayground
GameMenu {
    DiplomacyMenu {
        button!(InvadeCountry)
        button!(LaunchMissils)
        button!(Abdicate)
    }
    CraftMenu {
        button!(Craft(ConveyorBelt))
        button!(Craft(Assembler))
        button!(Craft(Chest))
    }
    DebugMenu {
        button!(PrintHello)
        button!(PrintGoodbye)
    }
    InventoryMenu {
        InventoryWindow(rules(100%, 100%)) {
            InventoryScrollArea(marker(Inventory) root column rules(100%, 1*))
        }
    }
}
```

Inventory item descriptions could be very complex. We could use the `dsl!` macro.
But chirp files are hot-reloadable, and when prototyping, we'd rather be able
to hot reload.

So let's create a new chirp file:

```rust,no_run,noplayground
// file <inventory_item.chirp>
InventoryItem(row rules(100%, 1.1*)) {
    ItemPreview(marker(ItemPreview) width(50px))
    ItemDescription(marker(ItemDescription))
}
```

Then, in a system, spawn one such scene per new inventory item:

```rust,no_run,noplayground
fn spawn_inventory(
    mut cmds: Commands,
    inventory: Query<(Entity, Option<&Children>), With<Inventory>>,
    items: Query<(), (With<Item>, Without<Handle<Chirp>>)>,
    asset_server: Res<AssetServer>,
) {
    let Ok((inventory, items)) = inventory.get_single() else {
        return;
    };
    for item in items.iter().flatten() {
        let mut cmds = cmds.entity(item);
        if items.contains(item) {
            cmds.insert(ChirpBundle(serv.load("inventory_item.chirp")))
        }
    }
}
```


[^1]: That I completely made up; No need for a survey, just look into the
dictionary.

[^2]: Actually not, as the rest of this section will demonstrate :P

[`bevy_framepace`]: https://crates.io/crates/bevy_framepace
[`bevy_mod_picking`]: https://crates.io/crates/bevy_mod_picking
[`On`]: https://docs.rs/bevy_mod_picking/latest/bevy_mod_picking/prelude/struct.On.html
[cuicui_richtext]: https://github.com/nicopap/cuicui/tree/main/richtext