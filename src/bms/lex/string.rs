// winnowに書いてあったコードのコピペ
use winnow::ascii::multispace1;
use winnow::combinator::alt;
use winnow::combinator::repeat;
use winnow::combinator::{delimited, preceded};
use winnow::error::{FromExternalError, ParserError};
use winnow::prelude::*;
use winnow::token::{take_till, take_while};

pub(crate) fn escaped_string<'a, E>(input: &mut &'a str) -> ModalResult<String, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let build_string = repeat(0.., parse_fragment).fold(
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );
    delimited('"', build_string, '"').parse_next(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

fn parse_fragment<'a, E>(input: &mut &'a str) -> ModalResult<StringFragment<'a>, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        parse_literal.map(StringFragment::Literal),
        parse_escaped_char.map(StringFragment::EscapedChar),
        parse_escaped_whitespace.value(StringFragment::EscapedWS),
    ))
    .parse_next(input)
}

fn parse_literal<'a, E: ParserError<&'a str>>(
    input: &mut &'a str,
) -> ModalResult<&'a str, E> {
    let not_quote_slash = take_till(1.., ['"', '\\']);

    not_quote_slash
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

fn parse_escaped_char<'a, E>(input: &mut &'a str) -> ModalResult<char, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    preceded(
        '\\',
        alt((
            parse_unicode,
            'n'.value('\n'),
            'r'.value('\r'),
            't'.value('\t'),
            'b'.value('\u{08}'),
            'f'.value('\u{0C}'),
            '\\'.value('\\'),
            '/'.value('/'),
            '"'.value('"'),
        )),
    )
    .parse_next(input)
}

fn parse_unicode<'a, E>(input: &mut &'a str) -> ModalResult<char, E>
where
    E: ParserError<&'a str>
        + FromExternalError<&'a str, std::num::ParseIntError>,
{
    let parse_hex = take_while(1..=6, |c: char| c.is_ascii_hexdigit());

    let parse_delimited_hex = preceded('u', delimited('{', parse_hex, '}'));

    let parse_u32 =
        parse_delimited_hex.try_map(move |hex| u32::from_str_radix(hex, 16));

    parse_u32.verify_map(std::char::from_u32).parse_next(input)
}

fn parse_escaped_whitespace<'a, E: ParserError<&'a str>>(
    input: &mut &'a str,
) -> ModalResult<&'a str, E> {
    preceded('\\', multispace1).parse_next(input)
}
