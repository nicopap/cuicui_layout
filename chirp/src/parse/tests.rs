use super::*;

#[test]
fn invalid_split_0_1() {}
#[test]
fn valid_split_str() {
    use split::split;

    let (hello, world, hi) = (r#""hello""#, r#""world""#, r#""hi""#);
    assert_eq!(split::<1>(r#"("")"#), Ok([r#""""#]));
    assert_eq!(split::<1>(r#"("'")"#), Ok([r#""'""#]));
    assert_eq!(split::<1>(r#"(",")"#), Ok([r#"",""#]));
    assert_eq!(split::<1>(r#"("hello")"#), Ok([hello]));
    assert_eq!(split::<1>(r#"("hello,world")"#), Ok([r#""hello,world""#]));
    assert_eq!(
        split::<1>(r#"("hello\\,world")"#),
        Ok([r#""hello\\,world""#])
    );
    assert_eq!(split::<1>(r#"("hello",)"#), Ok([hello]));
    assert_eq!(split::<1>(r#"("hello"  ,  )"#), Ok([hello]));
    assert_eq!(split::<1>(r#"(   "hello")"#), Ok([hello]));
    assert_eq!(split::<1>(r#"("hello"   )"#), Ok([hello]));
    assert_eq!(split::<2>(r#"("hello","world")"#), Ok([hello, world]));
    assert_eq!(split::<2>(r#"(  "hello", "world"  )"#), Ok([hello, world]));
    assert_eq!(split::<2>(r#"("hello"  , "world")"#), Ok([hello, world]));
    assert_eq!(split::<2>(r#"("hello","world",)"#), Ok([hello, world]));

    let arg3 = r#"("hello", "world", "hi")"#;
    let arg26 = r#"("a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", )"#;
    let arg26_eq = [
        r#""a""#, r#""b""#, r#""c""#, r#""d""#, r#""e""#, r#""f""#, r#""g""#, r#""h""#, r#""i""#,
        r#""j""#, r#""k""#, r#""l""#, r#""m""#, r#""n""#, r#""o""#, r#""p""#, r#""q""#, r#""r""#,
        r#""s""#, r#""t""#, r#""u""#, r#""v""#, r#""w""#, r#""x""#, r#""y""#, r#""z""#,
    ];
    assert_eq!(split::<3>(arg3), Ok([hello, world, hi]));
    assert_eq!(split::<26>(arg26), Ok(arg26_eq));
}
#[test]
fn valid_str_escape() {
    use split::split;

    // TODO(bug): Test the from_str impl.
    assert_eq!(split::<1>(r#"("hello")"#), Ok([r#""hello""#]));
    assert_eq!(split::<1>(r#"("hello world")"#), Ok([r#""hello world""#]));
    assert_eq!(split::<1>(r#"("hello\"world")"#), Ok([r#""hello\"world""#]));
    assert_eq!(split::<1>(r#"("hello\\world")"#), Ok([r#""hello\\world""#]));

    let escape_bs_qu = r#"("hello\\\\\"world")"#; // hello\\"world
    let bs_edge = r#"("\\hello world\\")"#; // \hello world\
    let escape_quotes = r#"("\'hello world\"")"#; // 'hello world"
    let edge_space = r#"("  hello world\"")"#; // |  hello world"|
    assert_eq!(split::<1>(escape_bs_qu), Ok([r#""hello\\\\\"world""#]));
    assert_eq!(split::<1>(bs_edge), Ok([r#""\\hello world\\""#]));
    assert_eq!(split::<1>(escape_quotes), Ok([r#""\'hello world\"""#]));
    assert_eq!(split::<1>(edge_space), Ok([r#""  hello world\"""#]));
}
#[test]
fn valid_split_tt() {
    use split::split;

    let fn_call = "(pct(100), px(34))";
    let fn_call_ws = "(  pct (\"hi\") , px   ( 34 ) )";
    let ident = "( hello )";
    let path = "(hello::world)";
    let nested_comma = "( hello( 10, 34, 14 ), world( 0x12 ,0x4 ) , )";
    let ctor = "( Rust::Paths { are: 10, cool: \"cool\" } , std::lib::env(32, 5))";
    assert_eq!(split::<2>(fn_call), Ok(["pct(100)", "px(34)"]));
    assert_eq!(split::<2>(fn_call_ws), Ok(["pct (\"hi\")", "px   ( 34 )"]));
    assert_eq!(split::<1>(ident), Ok(["hello"]));
    assert_eq!(split::<1>(path), Ok(["hello::world"]));
    assert_eq!(
        split::<2>(nested_comma),
        Ok(["hello( 10, 34, 14 )", "world( 0x12 ,0x4 )"])
    );
    assert_eq!(
        split::<2>(ctor),
        Ok([
            "Rust::Paths { are: 10, cool: \"cool\" }",
            "std::lib::env(32, 5)"
        ])
    );
}
#[test]
fn valid_split_0_1() {
    use split::split;

    let empty = r#""#;
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

    assert_eq!(split::<0>(empty), Ok([]));
    assert_eq!(split::<0>(empty_paren), Ok([]));
    assert_eq!(split::<0>(ws_paren), Ok([]));
    assert_eq!(split::<1>(str_1arg), Ok([r#""arg1""#]));
    assert_eq!(split::<1>(int_1arg), Ok(["1"]));
    assert_eq!(split::<1>(str_1arg_trailcomma), Ok([r#""arg1""#]));
    assert_eq!(split::<1>(int_1arg_trailcomma), Ok(["1"]));
    assert_eq!(split::<1>(str_1arg_ws), Ok([r#""arg1""#]));
    assert_eq!(split::<1>(int_1arg_ws), Ok(["1"]));
    assert_eq!(split::<1>(str_1arg_trailcomma_ws), Ok([r#""arg1""#]));
    assert_eq!(split::<1>(int_1arg_trailcomma_ws), Ok(["1"]));
}
