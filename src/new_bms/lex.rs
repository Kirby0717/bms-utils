use super::token::*;
use regex::Regex;
use std::sync::LazyLock;

type Result<T> = ::std::result::Result<T, String>;

macro_rules! static_regex {
    ($ident:ident=$regex:expr) => {
        static $ident: LazyLock<Regex> =
            LazyLock::new(|| Regex::new($regex).unwrap());
    };
}
macro_rules! command {
    ($ident:ident, $($regex:expr),* ) => {
        static_regex!($ident = concat!(r"^(?i:", stringify!($ident), r")", $($regex),*));
    };
    ($ident:ident) => {
        static_regex!($ident = concat!(r"^(?i:", stringify!($ident), r")"));
    };
    ($ident:ident as $command:ident, $($regex:expr),* ) => {
        static_regex!($ident = concat!(r"^(?i:", stringify!($command), r")", $($regex),*));
    };
    ($ident:ident as $command:ident) => {
        static_regex!($ident = concat!(r"^(?i:", stringify!($command), r")"));
    };
}
macro_rules! space {
    () => {
        r"\s"
    };
}
macro_rules! space0 {
    () => {
        r"\s*"
    };
}
macro_rules! space1 {
    () => {
        r"\s+"
    };
}
macro_rules! rest {
    () => {
        rest!("str")
    };
    ($ident:expr) => {
        concat!(r"(?<", $ident, r">.*)$")
    };
}
macro_rules! quote {
    () => {
        quote!("str")
    };
    ($ident:expr) => {
        concat!("\"(?<", $ident, ">.*)\"")
    };
}
macro_rules! channel {
    () => {
        channel!("ch")
    };
    ($ident:expr) => {
        concat!(r"(?<", $ident, r">[[:alnum:]]{2})")
    };
}
macro_rules! int {
    () => {
        int!("n")
    };
    ($ident:expr) => {
        concat!(r"(?<", $ident, r">-?[[:digit:]]+)")
    };
}
macro_rules! uint {
    () => {
        uint!("n")
    };
    ($ident:expr) => {
        concat!(r"(?<", $ident, r">[[:digit:]]+)")
    };
    ($ident:expr, $rep:expr) => {
        concat!(r"(?<", $ident, r">[[:digit:]]{", stringify!($rep), "})")
    };
}
macro_rules! float {
    () => {
        float!("n")
    };
    ($ident:expr) => {
        concat!(r"(?<", $ident, r">-?[[:digit:]]+(.[[:digit:]]*)?)")
    };
}
macro_rules! command_rest {
    ($ident:ident) => {
        command!($ident, space!(), rest!())
    };
}
macro_rules! command_quote {
    ($ident:ident) => {
        command!($ident, space1!(), quote!())
    };
    ($ident:ident as $command:ident) => {
        command!($ident as $command, space1!(), quote!())
    };
}
macro_rules! command_quote_rest {
    ($command:ident, $quote:ident) => {
        command_quote!($quote as $command);
        command_rest!($command);
    };
}
macro_rules! command_int {
    ($ident:ident) => {
        command!($ident, space1!(), int!())
    };
}
macro_rules! command_uint {
    ($ident:ident) => {
        command!($ident, space1!(), uint!())
    };
}
macro_rules! command_float {
    ($ident:ident) => {
        command!($ident, space1!(), float!())
    };
}

pub(super) fn lex(input: &str) -> Result<Option<Token>> {
    static_regex!(
        SHARP_COMMAND = concat!("^", space0!(), "#", space0!(), rest!("input"))
    );
    static_regex!(
        PERCENT_COMMAND =
            concat!("^", space0!(), "%", space0!(), rest!("input"))
    );
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
        MAIN_DATA = concat!(
            uint!("measure", 3),
            channel!(),
            space0!(),
            ":",
            rest!("input")
        )
    );
    command_uint!(PLAYER);
    command_int!(RANK);
    command_float!(DEFEXRANK);
    command!(EXRANK, channel!(), space1!(), float!());
    command_float!(TOTAL);
    command_float!(VOLWAV);
    command_rest!(STAGEFILE);
    command_rest!(BANNER);
    command_rest!(BACKBMP);
    command_rest!(CHARFILE);
    command_int!(PLAYLEVEL);
    command_int!(DIFFICULTY);
    command_quote_rest!(TITLE, TITLE_QUOTE);
    command_rest!(SUBTITLE);
    command_rest!(MAKER);
    command_rest!(GENRE);
    command_quote_rest!(COMMENT, COMMENT_QUOTE);
    static_regex!(
        TEXT_SONG =
            concat!(r"^(?i:(TEXT)|(SONG))", channel!(), space1!(), quote!())
    );
    command_rest!(PATH_WAV);
    command_float!(BPM);
    static_regex!(
        EXBPM =
            concat!(r"^(?i:(BPM)|(EXBPM))", channel!(), space1!(), float!())
    );
    command_float!(BASEBPM);
    command!(STOP, channel!(), space1!(), float!());
    command!(
        STP,
        space1!(),
        uint!("x", 3),
        space0!(),
        ".",
        space0!(),
        uint!("y", 3),
        space1!(),
        float!("z")
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
    else if let Some(cap) = VOLWAV.captures(input) {
        Command(VolumeWav(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = STAGEFILE.captures(input) {
        Command(StageFile(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = BANNER.captures(input) {
        Command(Banner(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = BACKBMP.captures(input) {
        Command(BackBmp(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = CHARFILE.captures(input) {
        Command(CharacterFile(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = PLAYLEVEL.captures(input) {
        Command(PlayLevel(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = DIFFICULTY.captures(input) {
        Command(Difficulty(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) =
        TITLE_QUOTE.captures(input).or(TITLE.captures(input))
    {
        Command(Title(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = SUBTITLE.captures(input) {
        Command(SubTitle(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = MAKER.captures(input) {
        Command(Maker(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = GENRE.captures(input) {
        Command(Genre(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) =
        COMMENT_QUOTE.captures(input).or(COMMENT.captures(input))
    {
        Command(Comment(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = TEXT_SONG.captures(input) {
        Command(Text(
            cap["ch"].parse().unwrap(),
            cap["str"].parse().unwrap(),
        ))
    }
    else if let Some(cap) = PATH_WAV.captures(input) {
        Command(PathWav(cap["str"].parse().unwrap()))
    }
    else if let Some(cap) = BPM.captures(input) {
        Command(Bpm(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = EXBPM.captures(input) {
        Command(ExBpm(cap["ch"].parse().unwrap(), cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = BASEBPM.captures(input) {
        Command(BaseBpm(cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = STOP.captures(input) {
        Command(Stop(cap["ch"].parse().unwrap(), cap["n"].parse().unwrap()))
    }
    else if let Some(cap) = STP.captures(input) {
        Command(Stp(
            cap["x"].parse().unwrap(),
            cap["y"].parse().unwrap(),
            cap["z"].parse().unwrap(),
        ))
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
