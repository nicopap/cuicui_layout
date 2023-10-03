use winnow::combinator::delimited;

use super::*;

fn split_tt(str_input: &'static str) -> Vec<&'static str> {
    let input = Input::new(str_input.as_bytes(), ());
    let (lparen, rparen, comma) = (tokens::Lparen, tokens::Rparen, tokens::Comma);
    let parsed = delimited(lparen, sep(many_tts::<true>), (opt(comma), rparen))
        .parse(input)
        .unwrap();
    parsed
        .into_iter()
        .map(|arg| {
            let range = arg.start as usize..arg.end as usize;
            &str_input[range]
        })
        .collect()
}

#[test]
fn valid_split_str() {
    let (hello, world, hi) = (r#""hello""#, r#""world""#, r#""hi""#);
    assert_eq!(split_tt(r#"("")"#), vec![r#""""#]);
    assert_eq!(split_tt(r#"("'")"#), vec![r#""'""#]);
    assert_eq!(split_tt(r#"(",")"#), vec![r#"",""#]);
    assert_eq!(split_tt(r#"("hello")"#), vec![hello]);
    assert_eq!(split_tt(r#"("hello,world")"#), vec![r#""hello,world""#]);
    assert_eq!(split_tt(r#"("hello\\,world")"#), vec![r#""hello\\,world""#]);
    assert_eq!(split_tt(r#"("hello",)"#), vec![hello]);
    assert_eq!(split_tt(r#"("hello"  ,  )"#), vec![hello]);
    assert_eq!(split_tt(r#"(   "hello")"#), vec![hello]);
    assert_eq!(split_tt(r#"("hello"   )"#), vec![hello]);
    assert_eq!(split_tt(r#"("hello","world")"#), vec![hello, world]);
    assert_eq!(split_tt(r#"(  "hello", "world"  )"#), vec![hello, world]);
    assert_eq!(split_tt(r#"("hello"  , "world")"#), vec![hello, world]);
    assert_eq!(split_tt(r#"("hello","world",)"#), vec![hello, world]);

    let arg3 = r#"("hello", "world", "hi")"#;
    let arg26 = r#"("a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", )"#;
    let arg26_eq = vec![
        r#""a""#, r#""b""#, r#""c""#, r#""d""#, r#""e""#, r#""f""#, r#""g""#, r#""h""#, r#""i""#,
        r#""j""#, r#""k""#, r#""l""#, r#""m""#, r#""n""#, r#""o""#, r#""p""#, r#""q""#, r#""r""#,
        r#""s""#, r#""t""#, r#""u""#, r#""v""#, r#""w""#, r#""x""#, r#""y""#, r#""z""#,
    ];
    assert_eq!(split_tt(arg3), vec![hello, world, hi]);
    assert_eq!(split_tt(arg26), arg26_eq);
}
#[test]
fn valid_str_escape() {
    // TODO(bug): Test the from_str impl.
    assert_eq!(split_tt(r#"("hello")"#), vec![r#""hello""#]);
    assert_eq!(split_tt(r#"("hello world")"#), vec![r#""hello world""#]);
    assert_eq!(split_tt(r#"("hello\"world")"#), vec![r#""hello\"world""#]);
    assert_eq!(split_tt(r#"("hello\\world")"#), vec![r#""hello\\world""#]);

    let escape_bs_qu = r#"("hello\\\\\"world")"#; // hello\\"world
    let bs_edge = r#"("\\hello world\\")"#; // \hello world\
    let escape_quotes = r#"("\'hello world\"")"#; // 'hello world"
    let edge_space = r#"("  hello world\"")"#; // |  hello world"|

    assert_eq!(split_tt(escape_bs_qu), vec![r#""hello\\\\\"world""#]);
    assert_eq!(split_tt(bs_edge), vec![r#""\\hello world\\""#]);
    assert_eq!(split_tt(edge_space), vec![r#""  hello world\"""#]);
    assert_eq!(split_tt(escape_quotes), vec![r#""\'hello world\"""#]);
}
#[test]
fn valid_split_tt() {
    let fn_call = "(pct(100), px(34))";
    let fn_call_ws = "(  pct (\"hi\") , px   ( 34 ) )";
    let ident = "( hello )";
    let path = "(hello::world)";
    let nested_comma = "( hello( 10, 34, 14 ), world( 0x12 ,0x4 ) , )";
    let ctor = "( Rust::Paths { are: 10, cool: \"cool\" } , std::lib::env(32, 5))";
    let weird_expr = r#"("\\\"hello", 10 - 32/4,  foo::Bar( method  ))"#;
    let comments = "(\n\
    \"Some text\", // With trailing comments\n\
    1334 + 420 - 0xdeadbeef, // And some maths\n\
    'Including single-quote strings', // more comments\n\
)";
    assert_eq!(split_tt(fn_call), vec!["pct(100)", "px(34)"]);
    assert_eq!(split_tt(fn_call_ws), vec!["pct (\"hi\")", "px   ( 34 )"]);
    assert_eq!(split_tt(ident), vec!["hello"]);
    assert_eq!(split_tt(path), vec!["hello::world"]);
    assert_eq!(
        split_tt(nested_comma),
        vec!["hello( 10, 34, 14 )", "world( 0x12 ,0x4 )"]
    );
    assert_eq!(
        split_tt(ctor),
        vec![
            "Rust::Paths { are: 10, cool: \"cool\" }",
            "std::lib::env(32, 5)"
        ]
    );
    assert_eq!(
        split_tt(weird_expr),
        vec![r#""\\\"hello""#, "10 - 32/4", "foo::Bar( method  )",]
    );
    assert_eq!(
        split_tt(comments),
        vec![
            r#""Some text""#,
            "1334 + 420 - 0xdeadbeef",
            "'Including single-quote strings'",
        ]
    );
}
#[test]
fn valid_split_0_1() {
    let empty_paren = r#"()"#;
    let ws_paren = r#"(   )"#;
    let str_1arg = r#"("arg1")"#;
    let int_1arg = r#"(1)"#;
    let str_1arg_trailcomma = r#"("arg1",)"#;
    let int_1arg_trailcomma = r#"(1,)"#;
    let str_1arg_ws = r#"(    "arg1")"#;
    let int_1arg_ws = r#"(1      )"#;
    let str_1arg_trailcomma_ws = r#"("arg1"  , )"#;
    let int_1arg_trailcomma_ws = r#"(  1,     )"#;

    assert_eq!(split_tt(empty_paren), Vec::<&'static str>::new());
    assert_eq!(split_tt(ws_paren), Vec::<&'static str>::new());
    assert_eq!(split_tt(str_1arg), vec![r#""arg1""#]);
    assert_eq!(split_tt(int_1arg), vec!["1"]);
    assert_eq!(split_tt(str_1arg_trailcomma), vec![r#""arg1""#]);
    assert_eq!(split_tt(int_1arg_trailcomma), vec!["1"]);
    assert_eq!(split_tt(str_1arg_ws), vec![r#""arg1""#]);
    assert_eq!(split_tt(int_1arg_ws), vec!["1"]);
    assert_eq!(split_tt(str_1arg_trailcomma_ws), vec![r#""arg1""#]);
    assert_eq!(split_tt(int_1arg_trailcomma_ws), vec!["1"]);
}
