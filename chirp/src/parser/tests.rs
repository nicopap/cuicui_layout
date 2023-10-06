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
    fn set_method(&mut self, name: &[u8], args: &Arguments) {
        let hier = self.hierarchy.get_index_mut(&self.current);
        let name = String::from_utf8_lossy(name).to_string();
        let args = args.to_string();
        hier.methods.insert(name, args);
    }
}
#[derive(Debug)]
struct TestInterpreter(TestState);
impl TestInterpreter {
    fn new() -> Self {
        TestInterpreter(TestState::new())
    }
}

impl<'i, 'a> Interpreter<'i, 'a> for TestInterpreter {
    fn code(&mut self, (code, range): (&[u8], Span)) {
        let current = self.0.hierarchy.get_index_mut(&self.0.current);
        current.insert_code(code, range);
    }

    fn set_name(&mut self, (name, _): Name) {
        self.0.set_name(name);
    }

    fn complete_children(&mut self) {
        if let Some(last) = self.0.current.last_mut() {
            *last += 1;
        }
    }

    fn method(&mut self, (name, _): Name, args: &Arguments) {
        self.0.set_method(name, args);
    }

    fn start_children(&mut self) {
        self.0.current.push(0);
    }

    fn get_template(&mut self, _name: Name<'i>) -> Option<FnIndex<'a>> {
        todo!()
    }

    fn import(&mut self, _name: Name<'i>, _alias: Option<Name<'i>>) {
        todo!()
    }

    fn register_fn(&mut self, _name: Name<'i>, _index: FnIndex<'a>) {
        todo!()
    }
}
#[track_caller]
fn interpret(input: &str) -> Hier {
    let input = Input::new(input.as_bytes(), ());
    let chirp_file = super::chirp_file(input).unwrap();
    let mut state = TestInterpreter::new();
    let file = ChirpFile::new(input, chirp_file.as_ref());
    file.interpret(&mut state);
    state.0.hierarchy
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
