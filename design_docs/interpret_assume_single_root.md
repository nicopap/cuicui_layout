# How to interpret chirp files when we know we exactly have 1 root?

Interpreter:

```rust
struct Interp {
  commands: Commands,
  parents: Vec<Entity>,
  dsl: Dsl,
}
impl Interp {
  fn new(commands: Commands) -> Self {
    Interp {
      commands,
      parents: Vec::new(),
      dsl: Dsl::default(),
    }
  }
  fn call_method(&mut self, name, args) {
    self.dsl.method(name, args);
  }
  fn spawn_current(&mut self) -> Entity {
    let mut new_entity = self.commands.spawn_empty();
    if let Some(parent) = self.parents.last().copied() {
      new_entity.set_parent(parent);
    }
    self.dsl.insert(&mut entity)
  }
  fn spawn_with_children(&mut self) {
    let inserted = self.spawn_current();
    self.parents.push(inserted);
  }
  fn leave_children(&mut self) {
    self.parents.pop();
  }
}
```

Problem: We want to pass a `&mut EntityCommands` to the `Interp`.

We want the root entity to be spawned on the provided `EntityCommands`.

```rust
struct Interp {
  commands: Commands,
  current: Option<Entity>,
  parents: Vec<Entity>,
  dsl: Dsl,
}
impl Interp {
  fn new(entity_commands: &mut EntityCommands) -> Self {
    let current = entity_commands.id();
    Interp {
      commands: entity_commands.commands(),
      current: Some(current),
      parents: Vec::new(),
      dsl: Dsl::default(),
    }
  }
  fn call_method(&mut self, name, args) {
    self.dsl.method(name, args);
  }
  fn spawn_current(&mut self) -> Entity {
    let mut new_entity = match self.current.take() {
      None => self.commands.spawn_empty(),
      Some(entity) => self.commands.entity(entity),
    };
    if let Some(parent) = self.parents.last().copied() {
      new_entity.set_parent(parent);
    }
    self.dsl.insert(&mut entity)
  }
  fn spawn_with_children(&mut self) {
    let inserted = self.spawn_current();
    self.parents.push(inserted);
  }
  fn leave_children(&mut self) {
    self.parents.pop();
  }
}
```
