use std::cell::RefCell;

use bevy::utils::HashMap;

use super::*;

macro_rules! hier {
    (@inner [$_:ident]) => {};
    (@inner [$acc:ident] $name:ident ($( $method:ident $methd_arg:literal )*)
        {$($inner:tt)*} $($($rem:tt)+)?
    ) => {
        $acc.push(Hier {
            name: stringify!($name).to_string(),
            methods: {
                let mut r = HashMap::new();
                $(r.insert(
                    stringify!($method).to_string(),
                    $methd_arg.to_string(),
                );)*
                r
            },
            code: HashMap::new(),
            children: {
                let mut r = Vec::new();
                hier!(@inner [r] $($inner)*);
                r
            },
        })
        $(; hier!(@inner $($rem)+))?
    };
    ($name:ident ($( $method:ident $methd_arg:literal )*) {$($inner:tt)*}) => {{
        #[allow(unused_mut)]
        let x = Hier {
            name: stringify!($name).to_string(),
            methods: {
                let mut r = HashMap::new();
                $(r.insert(
                    stringify!($method).to_string(),
                    $methd_arg.to_string(),
                );)*
                r
            },
            code: HashMap::new(),
            children: {
                let mut r = Vec::new();
                hier!(@inner [r] $($inner)*);
                r
            },
        };
        x
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Hier {
    name: String,
    methods: HashMap<String, String>,
    code: HashMap<String, Span>,
    children: Vec<Hier>,
}
impl Hier {
    fn insert_code(&mut self, code: &[u8], range: Span) {
        let utf8 = String::from_utf8_lossy(code).to_string();
        self.code.insert(utf8, range);
    }
    fn get_index_mut<'a>(&'a mut self, path: &[usize]) -> &'a mut Self {
        let Some((head, tail)) = path.split_first() else {
            return self;
        };
        if let Some(child) = self.children.get_mut(*head) {
            return child.get_index_mut(tail);
        } else {
            panic!("bad")
        }
    }

    fn new(name: String) -> Hier {
        Hier {
            name,
            methods: HashMap::new(),
            code: HashMap::new(),
            children: Vec::new(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct TestState {
    hierarchy: Hier,
    current: Vec<usize>,
}
impl TestState {
    fn new() -> Self {
        TestState {
            hierarchy: Hier::new("".to_string()),
            current: vec![],
        }
    }
    fn set_name(&mut self, name: &[u8]) {
        let hier = self.hierarchy.get_index_mut(&self.current);
        hier.name = String::from_utf8_lossy(name).to_string();
    }
    fn set_method(&mut self, name: &[u8], args: &[u8]) {
        let hier = self.hierarchy.get_index_mut(&self.current);
        let name = String::from_utf8_lossy(name).to_string();
        let args = String::from_utf8_lossy(args).to_string();
        hier.methods.insert(name, args);
    }
}
#[derive(Debug)]
struct TestInterpreter(RefCell<TestState>);
impl TestInterpreter {
    fn new() -> Self {
        TestInterpreter(RefCell::new(TestState::new()))
    }
}

impl Itrp for &'_ TestInterpreter {
    fn code(&self, (code, range): (&[u8], Span)) {
        let state = &mut *self.0.borrow_mut();
        let current = state.hierarchy.get_index_mut(&state.current);
        current.insert_code(code, range);
    }

    fn set_name(&self, _span: Span, name: &[u8]) {
        self.0.borrow_mut().set_name(name);
    }

    fn complete_children(&self) {
        let state = &mut *self.0.borrow_mut();
        state.current.pop();
        if let Some(last) = state.current.last_mut() {
            *last += 1;
        }
    }

    fn method(&self, name: &[u8], _: Span, args: &[u8], _: Span) {
        self.0.borrow_mut().set_method(name, args);
    }

    fn spawn_with_children(&self) {
        self.0.borrow_mut().current.push(0);
    }
}
#[track_caller]
fn interpret(input: &str) -> Hier {
    let state = TestInterpreter::new();
    let stateful_input = Input::new(input.as_bytes(), &state);
    super::chirp_document(stateful_input).unwrap();
    state.0.into_inner().hierarchy
}

#[test]
fn simple() {
    let actual = interpret("Name()");
    assert_eq!(actual, hier!(Name() {}));
}
#[test]
fn padded() {
    let actual = interpret("  Name()  ");
    assert_eq!(actual, hier!(Name() {}));
}
#[test]
fn very_padded() {
    let actual = interpret("  Name     (     )  ");
    assert_eq!(actual, hier!(Name() {}));
}
#[test]
fn comment_padded() {
    let actual = interpret(
        r#"// Foo bar
Name     (     )
// final comment
"#,
    );
    assert_eq!(actual, hier!(Name() {}));
}
#[test]
fn with_method() {
    let actual = interpret("Name(method(10))");
    assert_eq!(actual, hier!(Name(method "(10)") {}));
    let actual = interpret("Name(method  (10)  )");
    assert_eq!(actual, hier!(Name(method "(10)") {}));
}
