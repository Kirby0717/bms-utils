use super::token::*;
use regex::Regex;
use std::sync::LazyLock;

type Result<T> = ::std::result::Result<T, String>;

macro_rules! static_regex {
    ($name:ident=$regex:expr) => {
        static $name: LazyLock<Regex> =
            LazyLock::new(|| Regex::new($regex).unwrap());
    };
}

pub(super) fn lex(input: &str) -> Result<Option<Token>> {
    static_regex!(SHARP_COMMAND = r"^\s*#\s*(?<input>.*)$");
    static_regex!(PERCENT_COMMAND = r"^\s*%\s*(?<input>.*)$");
    let token = if let Some(cap) = SHARP_COMMAND.captures(input) {
        Some(sharp_command(&cap["input"])?)
    }
    else if let Some(cap) = PERCENT_COMMAND.captures(input) {
        Some(percent_command(&cap["input"])?)
    }
    else {
        None
    };
    Ok(token)
}

fn sharp_command(input: &str) -> Result<Token> {
    use super::token::Command::*;
    use Token::*;

    static_regex!(
        MAIN_DATA = r"^(?<measure>[[:digit:]]{3})(?<ch>[[:alnum:]]{2})\s*:(?<input>.*)$"
    );
    static_regex!(PLAYER = r"^(?i:PLAYER)\s+(?<n>-?[[:digit:]]+)");
    static_regex!(RANK = r"^(?i:RANK)\s+(?<n>-?[[:digit:]]+)");
    static_regex!(
        DEFEXRANK = r"^(?i:DEFEXRANK)\s+(?<n>-?[[:digit:]]+(.[[:digit:]]*)?)"
    );
    static_regex!(
        EXRANK = r"^(?i:EXRANK)(?<ch>[[:alnum:]]{2})\s+(?<n>-?[[:digit:]]+)"
    );
    static_regex!(
        TOTAL = r"^(?i:TOTAL)\s+(?<n>-?[[:digit:]]+(.[[:digit:]]*)?)"
    );
    let token = if let Some(cap) = MAIN_DATA.captures(input) {
        Command(MainData(
            cap["measure"].parse::<usize>().unwrap(),
            main_data(cap["ch"].parse().unwrap(), &cap["input"])?,
        ))
    }
    else if let Some(cap) = PLAYER.captures(input) {
        let n = cap["n"].parse().unwrap();
        if !(1..=4).contains(&n) {
            return Err(format!("PLAYERの不正な値{n}"));
        }
        Command(Player(n))
    }
    else if let Some(cap) = RANK.captures(input) {
        Command(Rank(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = DEFEXRANK.captures(input) {
        Command(DefExRank(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = EXRANK.captures(input) {
        Command(ExRank(
            cap["ch"].parse().unwrap(),
            cap["n"].parse().unwrap(),
        ))
    }
    else if let Some(cap) = TOTAL.captures(input) {
        Command(Total(cap["n"].parse().unwrap()))
    }
    else {
        //Command(Other(input.to_string()))
        return Err(String::new());
    };
    Ok(token)
}
fn main_data(channel: Channel, input: &str) -> Result<MainDataValue> {
    use MainDataValue::*;
    Err(String::new())
}

fn percent_command(input: &str) -> Result<Token> {
    Err(String::new())
}
