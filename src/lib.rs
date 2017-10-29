#[macro_use]
extern crate nom;
extern crate ordermap;

mod helper;

use helper::{buf_to_u32, buf_to_i64};
use ordermap::OrderMap;
use nom::{digit, IResult};


#[derive(Debug, PartialEq)]
pub struct Null;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Str(String),
    Int(i64),
}

type Array = OrderMap<Key, Value>;

#[derive(Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Str(String),
    Int(i64),
    Null,
    //    Double(f32),
    Array(Array)
}


named!(boolean<bool>,
    delimited!(
        tag!("b:"), alt!(tag!("0") => { |_| false } | tag!("1") => { |_| true } ), tag!(";")
    )
);

fn field_len<'a>(input: &'a [u8], pref: &str) -> IResult<&'a [u8], u32> {
    delimited!(input, tag!(pref), map!(call!(digit), buf_to_u32), tag!(":"))
}

named!(string<&str>,
    do_parse!(
        length: call!(field_len, "s:") >>
        res: delimited!(tag!("\""), take_str!(length), tag!("\";")) >>
        (res)
    )
);

// @TODO sign
named!(int<i64>, delimited!(tag!("i:"), map!(call!(digit), buf_to_i64), tag!(";")));

named!(null<()>, value!((), tag!("N;")));

named!(pub value<Value>,
    alt!(
        boolean => { |b|   Value::Bool(b)               } |
        string  => { |s|   Value::Str(String::from(s))  } |
        int     => { |i|   Value::Int(i)                } |
        null    => { |_|   Value::Null                  } |
        // double
        array   => { |a|   Value::Array(a)              }
    )
);

named!(key<Key>,
    alt!(
        string  => { |s|   Key::Str(String::from(s))  } |
        int     => { |i|   Key::Int(i)                }
    )
);

named!(keyval<(Key,Value)>, pair!(key, value));

named!(array<Array>,
    do_parse!(
        length: call!(field_len, "a:") >>
        res: map!(
            delimited!(
                tag!("{"),
                count!(keyval, length as usize),
                tag!("}")
            ), | tuple_vec | {
                let mut h: OrderMap<Key, Value> = OrderMap::new();
                for (k, v) in tuple_vec {
                  h.insert(k, v);
                }
                h
            }
        ) >>
        (res)
    )
);

#[test]
fn test_boolean() {
    assert_eq!(IResult::Done(&[][..], true), boolean(b"b:1;"));
    assert_eq!(IResult::Done(&[][..], false), boolean(b"b:0;"));
}

#[test]
fn test_string() {
    assert_eq!(IResult::Done(&[][..], "string"), string(br#"s:6:"string";"#));
    assert_eq!(IResult::Done(&[][..], "āžčģā"), string(r#"s:10:"āžčģā";"#.as_bytes()));
}

#[test]
fn test_int() {
    assert_eq!(IResult::Done(&[][..], 1), int(br#"i:1;"#));
//    assert_eq!(IResult::Done(&[][..], -1), int(br#"i:-1;"#));
    assert_eq!(IResult::Done(&[][..], 12), int(br#"i:12;"#));
}

#[test]
fn test_null() {
    assert_eq!(IResult::Done(&[][..], ()), null(b"N;"));
}


#[test]
fn test_key() {
    assert_eq!(IResult::Done(&[][..], Key::Str("string".to_owned())), key(br#"s:6:"string";"#));
    assert_eq!(IResult::Done(&[][..], Key::Int(1)), key(br#"i:1;"#));
}

#[test]
fn test_keyval() {
    assert_eq!(IResult::Done(&[][..], (Key::Int(1), Value::Int(1))), keyval(br#"i:1;i:1;"#));
}


#[test]
fn test_array() {
    let mut h = Array::new();
    h.insert(Key::Int(0), Value::Null);
    h.insert(Key::Int(1), Value::Null);
    assert_eq!(IResult::Done(&[][..], h), array(br#"a:2:{i:0;N;i:1;N;}"#));
}


#[test]
fn test_value() {
    assert_eq!(IResult::Done(&[][..], Value::Bool(false)), value(br#"b:0;"#));
    assert_eq!(IResult::Done(&[][..], Value::Str("string".to_owned())), value(br#"s:6:"string";"#));
    assert_eq!(IResult::Done(&[][..], Value::Int(1)), value(br#"i:1;"#));
    assert_eq!(IResult::Done(&[][..], Value::Null), value(b"N;"));

    let mut most_inner = Array::new();
    most_inner.insert(Key::Int(0), Value::Int(1));

    let mut inner = Array::new();
    inner.insert(Key::Int(0), Value::Array(most_inner));

    let mut outer = Array::new();
    outer.insert(Key::Int(0), Value::Array(inner));

    assert_eq!(IResult::Done(&[][..], Value::Array(outer)), value(br"a:1:{i:0;a:1:{i:0;a:1:{i:0;i:1;}}}"));
}
