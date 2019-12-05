//! possible refactor:
//!     - make Tokenizer a trait
//!     -
//!

use ipld_core::Token;
use nom;
use util::{
    parse_string, string_bytes, tag_bytes_end, tag_bytes_start, tag_comma, tag_link_key,
    tag_link_start, tag_map_end, tag_map_start, tag_semicolon,
};

/******************************************************************************
 * Main single token parser
 *****************************************************************************/

// TODO: wrap in ws!?
named!(pub lex_start<&[u8], Token>, alt!(primitive | compound_start));

named!(primitive<&[u8], Token>, alt!(
   bytes
   | link
   | boolean
   | null
   | number
   | string
));

named!(compound_start<&[u8], Token>, alt!(list_start | map_start));
named!(compound_end<&[u8], Token>, alt!(list_end | map_end));

/******************************************************************************
 * Raw token parsers
 *****************************************************************************/

/*
 * Null
 */
named_attr!(#[inline], null<&[u8], Token>, value!(Token::Null, util::tag_null));

/*
 * Boolean
 */
named_attr!(#[inline], boolean<&[u8], Token>, alt!(
    value!(Token::Bool(true), util::tag_true) |
    value!(Token::Bool(false), util::tag_false)
));

/*
 * Number
 */
named_attr!(#[inline], number<&[u8], Token>, alt!(float | integer));
named_attr!(#[inline], float<&[u8], Token>, map!(util::float_as_str, Token::FloatStr));
named_attr!(#[inline], integer<&[u8], Token>, alt!(util::signed | util::unsigned));

/*
 * String
 */
named_attr!(#[inline], string<&[u8], Token>, map!(util::parse_string, Token::StrRaw));

/*
 * Bytes
 */

named_attr!(#[inline], bytes<&[u8], Token>, do_parse!(
            tag_bytes_start >>
    _base:  string_bytes >>
            tag_semicolon >>
    bytes:  parse_string >>
            tag_bytes_end >>
            (Token::BytesStr(&bytes))
));

/*
 * List
 */
named_attr!(#[inline], list_start<&[u8], Token>, value!(Token::ListStart(None), util::tag_list_start));
named_attr!(#[inline], list_end<&[u8], Token>, value!(Token::ListEnd, util::tag_list_end));
// named_attr!(#[inline], list_element_separator<&[u8], ()>, value!((), util::tag_comma));

/*
 * Map
 */
named_attr!(#[inline], map_start<&[u8], Token>, value!(Token::MapStart(None), util::tag_map_start));
named_attr!(#[inline], map_end<&[u8], Token>, value!(Token::MapEnd, util::tag_map_end));
named_attr!(#[inline], map_key<&[u8], Token>, do_parse!(
    k:  alt!(integer | string) >>
        tag_semicolon >>
        (k)
));
// named_attr!(#[inline], map_element_separator<&[u8], ()>, value!((), util::tag_comma));

/*
 * Link
 */
named_attr!(#[inline], link<&[u8], Token>, do_parse!(
                tag_link_start >>
    cid_str:    parse_string >>
                tag_map_end >>
                (Token::LinkStr(cid_str))
));

#[allow(dead_code)]
mod util {
    use ipld_core::Token;
    use nom::{
        character::streaming::{alphanumeric1, digit1},
        number::streaming::recognize_float,
    };

    /****************************************/
    // string-related
    /****************************************/

    const QU: bool = false; // double quote       0x22
    const BS: bool = false; // backslash          0x5C
    const CT: bool = false; // control character  0x00 ... 0x1F
    const __: bool = true;

    static ALLOWED_CHARS: [bool; 256] = [
        //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ];

    // FIXME: support unicode characters
    fn is_string_char(c: u8) -> bool {
        ALLOWED_CHARS[c as usize]
    }

    named_attr!(#[inline], pub(crate) parse_string<&[u8], &str>, map!(
        string_bytes,
        |bytes| unsafe { std::str::from_utf8_unchecked(bytes) }
    ));

    named_attr!(#[inline], pub(crate) string_bytes<&[u8], &[u8]>, do_parse!(
            tag!(b"\"") >>
        s:  escaped!(take_while1!(is_string_char), '\\', one_of!("\"\\bfnrt")) >>
            tag!(b"\"") >>
            (s)
    ));

    /****************************************/
    // number-related
    /****************************************/

    named_attr!(#[inline], pub(crate) signed<&[u8], Token>, do_parse!(
                peek!(tag!(b"-")) >>
        token:  map!(int_as_str, Token::IntegerStr) >>
                (token)
    ));
    named_attr!(#[inline], pub(crate) unsigned<&[u8], Token>, do_parse!(
                opt!(tag!(b"+")) >>
        token:  map!(uint_as_str, Token::IntegerStr) >>
                (token)
    ));

    named_attr!(#[inline], pub(crate) float_as_str<&[u8], &str>, map!(
        recognize_float,
        |bytes| unsafe { std::str::from_utf8_unchecked(bytes) }
    ));

    named_attr!(#[inline], pub(crate) int_as_str<&[u8], &str>, map!(
        recognize!(preceded!(opt!(tag!(b"-")), digit1)),
        |bytes| unsafe { std::str::from_utf8_unchecked(bytes) }
    ));
    named_attr!(#[inline], pub(crate) uint_as_str<&[u8], &str>, map!(
        digit1,
        |bytes| unsafe { std::str::from_utf8_unchecked(bytes) }
    ));

    /****************************************/
    // whitespace, punctuation, tags
    /****************************************/

    named_attr!(#[inline], pub(crate) tag_null<&[u8], &[u8]>, tag!(b"null"));
    named_attr!(#[inline], pub(crate) tag_true<&[u8], &[u8]>, tag!(b"true"));
    named_attr!(#[inline], pub(crate) tag_false<&[u8], &[u8]>, tag!(b"false"));
    named_attr!(#[inline], pub(crate) tag_link_key<&[u8], &[u8]>, tag!(b"\"/\""));
    named_attr!(#[inline], pub(crate) tag_link_start<&[u8], ()>, do_parse!(
        tag_map_start >> tag_link_key >> tag_semicolon >> ()
    ));
    named_attr!(#[inline], pub(crate) tag_bytes_start<&[u8], ()>, do_parse!(
        tag_link_start >> tag_map_start >> ()
    ));
    named_attr!(#[inline], pub(crate) tag_bytes_end<&[u8], ()>, do_parse!(
        tag_map_end >> tag_map_end >> ()
    ));
    named_attr!(#[inline], pub(crate) tag_list_start<&[u8], &[u8]>, tag!(b"["));
    named_attr!(#[inline], pub(crate) tag_list_end<&[u8], &[u8]>, tag!(b"]"));
    named_attr!(#[inline], pub(crate) tag_map_start<&[u8], &[u8]>, tag!(b"{"));
    named_attr!(#[inline], pub(crate) tag_map_end<&[u8], &[u8]>, tag!(b"}"));

    named_attr!(#[inline], pub(crate) tag_comma<&[u8], Option<&[u8]>>, opt!(eat_separator!(b",")));
    named_attr!(#[inline], pub(crate) tag_semicolon<&[u8], &[u8]>, eat_separator!(b":"));

    // named!(whitespace<&[u8], Token>, ws!)
    // named!(esc_quote<&[u8]>, escaped!(b"\\\"", '\\', |_| "\""));
}

// #[cfg(test)]
// mod tests {
//     use crate::{encoder::to_vec, tokenizer::Tokenizer};
//     use ipld_core::{
//         multibase::{Base, Encodable},
//         Token,
//     };
//     use serde::Serialize;
//     use serde_bytes::ByteBuf;
//     use std::io::Write;

//     #[test]
//     fn test_null() {
//         let json = to_newlined_json_vec(None as Option<()>);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::Null, actual);
//     }

//     #[test]
//     fn test_boolean() {
//         let json = to_newlined_json_vec(true);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::Bool(true), actual);

//         let json = to_newlined_json_vec(false);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::Bool(false), actual);
//     }

//     #[test]
//     fn test_integer() {
//         let num: i128 = std::i128::MIN;
//         let json = to_newlined_json_vec(&num);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::IntegerStr(&format(num)), actual);

//         let num: u128 = std::u128::MAX;
//         let json = to_newlined_json_vec(&num);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::IntegerStr(&format(num)), actual);
//     }

//     #[test]
//     fn test_float() {
//         let num: f64 = 3.14159265358979323846264338327950288;
//         let json = to_newlined_json_vec(&num);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::FloatStr(&format(num)), actual);
//     }

//     #[test]
//     fn test_string() {
//         // simple string
//         let string = "hello world";
//         let json = to_newlined_json_vec(&string);
//         println!("1st: {:?}\nbytes: {:?}", format_vec(&json), &json);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::StrRaw(&string), actual);

//         // TODO FIXME: containing double quotes
//         let string = r#"world "double-quoted hello""#;
//         let json = to_newlined_json_vec(&string);
//         println!("2nd: {:?}\nbytes: {:?}", format_vec(&json), &json);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::StrRaw(&string), actual);

//         // utf8 emoji
//         let string = r#"i ❤ ny"#;
//         let json = to_newlined_json_vec(&string);
//         println!("3rd: {:?}\nbytes: {:?}", format_vec(&json), &json);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::StrRaw(&string), actual);

//         // TODO FIXME: containing raw unicode codepoint
//         let string = r#"i \u2764 ny"#;
//         let json = to_newlined_json_vec(&string);
//         println!("4th: {:?}\nbytes: {:?}", format_vec(&json), &json);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::StrRaw(&string), actual);
//     }

//     #[test]
//     fn test_bytes() {
//         let bytes = ByteBuf::from(vec![0, 1, 2, 3]);
//         let byte_str = bytes.encode(Base::Base64);
//         let json = to_newlined_json_vec(&bytes);
//         println!("1st: {}\nbytes: {:?}", format_vec(&json), &json);
//         let actual = Tokenizer::new(&json).next().unwrap();
//         assert_eq!(Token::BytesStr(&byte_str), actual);
//     }

//     // Encodes the type, then newline ends the byte vec
//     fn to_newlined_json_vec<T: Serialize>(t: T) -> Vec<u8> {
//         let mut vec = to_vec(&t).unwrap();
//         writeln!(&mut vec).unwrap();
//         vec
//     }

//     fn format<T: std::fmt::Display>(t: T) -> String {
//         format!("{}", t)
//     }

//     fn format_vec(v: &Vec<u8>) -> &str {
//         unsafe { std::str::from_utf8_unchecked(v) }
//     }
// }
