#![allow(dead_code)]

use super::token::*;
use Command::*;
use ControlFlow::*;
use winnow::{
    ascii::{Caseless, alphanumeric1, dec_int, dec_uint, digit1, float},
    combinator::{alt, dispatch, empty, opt, preceded, repeat, separated},
    error::ParserError,
    prelude::*,
    stream::AsChar,
    token::{any, rest, take_while},
};
mod string;
use string::escaped_string;

fn rest_string(input: &mut &str) -> ModalResult<String> {
    rest.map(|s: &str| s.to_string()).parse_next(input)
}
fn one_of_space(input: &mut &str) -> ModalResult<char> {
    any.verify(|c: &char| c.is_whitespace()).parse_next(input)
}
fn space0<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    take_while(0.., |c: char| c.is_whitespace()).parse_next(input)
}
fn space1<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    take_while(1.., |c: char| c.is_whitespace()).parse_next(input)
}
fn channel(input: &mut &str) -> ModalResult<Channel> {
    Ok(Channel::from(
        take_while(2, |c: char| c.is_ascii_alphanumeric()).parse_next(input)?,
        //winnow::ascii::alphanumeric1.parse_next(input)?
    ))
}
fn padded_uint<N: std::str::FromStr + std::default::Default>(
    input: &mut &str,
) -> ModalResult<N> {
    digit1
        .map(|s: &str| N::from_str(s).unwrap_or_default())
        .parse_next(input)
}
fn quoted_or_no_quote(input: &mut &str) -> ModalResult<String> {
    alt((preceded(space0, escaped_string), rest_string)).parse_next(input)
}

pub(crate) fn lex(input: &str) -> Vec<Token> {
    let mut r = vec![];
    for (line, mut input) in input.lines().enumerate() {
        match preceded(space0, command).parse_next(&mut input) {
            Ok(t) => {
                if t != Token::Comment {
                    r.push(t);
                }
            }
            Err(e) => {
                log::warn!("{}行の解析に失敗しました", line + 1);
                log::debug!("{e}");
            }
        }
    }
    r
}
fn command(input: &mut &str) -> ModalResult<Token> {
    if input.is_empty() {
        return Ok(Token::Comment);
    }
    dispatch! {any;
        '%' => percent_command,
        '#' => sharp_command,
        _ => empty.value(Token::Comment),
    }
    .parse_next(input)
}
fn percent_command(input: &mut &str) -> ModalResult<Token> {
    alt((url, email, other)).parse_next(input)
}
fn sharp_command(input: &mut &str) -> ModalResult<Token> {
    let parsers = [
        main_data,
        player,
        rank,
        def_ex_rank,
        ex_rank,
        total,
        volume_wav,
        stage_file,
        banner,
        back_bmp,
        character_file,
        play_level,
        difficulty,
        title,
        sub_title,
        artist,
        sub_artist,
        maker,
        genre,
        comment,
        text,
        song,
        path_wav,
        bpm,
        ex_bpm,
        base_bpm,
        stop,
        stp,
        ln_mode,
        ln_type,
        ln_object,
        oct_fp,
        option,
        change_option,
        wav,
        wav_command,
        ex_wav,
        cdda,
        midi_file,
        bmp,
        ex_bmp,
        bga,
        at_bga,
        poor_bga,
        switch_bga,
        argb,
        video_file,
        video_fps,
        video_colors,
        video_delay,
        movie,
        seek,
        ex_character,
        scroll,
        speed,
        preview,
        base62,
        random,
        set_random,
        end_random,
        r#if,
        else_if,
        r#else,
        end_if,
        switch,
        set_switch,
        end_switch,
        case,
        skip,
        default,
    ];
    for parser in parsers {
        if let Some(t) = opt(parser).parse_next(input)? {
            return Ok(t);
        }
    }
    other.parse_next(input)
}
const fn base36(s: &str) -> usize {
    Channel::new(s).to_base_36()
}
fn main_data(input: &mut &str) -> ModalResult<Token> {
    let (n, ch, _) = (
        take_while(3, AsChar::is_dec_digit)
            .map(|s: &str| s.parse::<usize>().unwrap()),
        channel.map(|ch| ch.to_base_36()),
        (space0, ":"),
    )
        .parse_next(input)?;

    let mut ch_vec = repeat(0.., preceded(space0, channel));
    let mut hex_vec = repeat(
        0..,
        preceded(
            space0,
            take_while(2, |c: char| c.is_hex_digit())
                .map(|s: &str| u8::from_str_radix(s, 16).unwrap()),
        ),
    );
    const BGM: usize = base36("01");
    const LENGTH: usize = base36("02");
    const BPM: usize = base36("03");
    const BGA: usize = base36("04");
    const BGA_POOR: usize = base36("06");
    const BGA_LAYER: usize = base36("07");
    const EX_BPM: usize = base36("08");
    const BGA_LAYER2: usize = base36("0A");
    const BGA_ALPHA: usize = base36("0B");
    const BGA_LAYER_ALPHA: usize = base36("0C");
    const BGA_LAYER2_ALPHA: usize = base36("0D");
    const BGA_POOR_ALPHA: usize = base36("0E");
    const NOTE_S: usize = base36("11");
    const NOTE_E: usize = base36("2Z");
    const INVISIBLE_NOTE_S: usize = base36("31");
    const INVISIBLE_NOTE_E: usize = base36("4Z");
    const LONG_NOTE_S: usize = base36("51");
    const LONG_NOTE_E: usize = base36("6Z");
    const TEXT: usize = base36("99");
    const BGA_ARGB: usize = base36("A1");
    const BGA_LAYER_ARGB: usize = base36("A2");
    const BGA_LAYER2_ARGB: usize = base36("A3");
    const BGA_POOR_ARGB: usize = base36("A4");
    const SWITCH_BGA: usize = base36("A5");
    const LANDMINE_S: usize = base36("D1");
    const LANDMINE_E: usize = base36("E9");
    const SCROLL: usize = base36("SC");
    const SPEED: usize = base36("SP");

    use MainDataValue::*;
    let data = match ch {
        BGA => Bga(ch_vec.parse_next(input)?),
        BGA_POOR => BgaPoor(ch_vec.parse_next(input)?),
        BGA_LAYER => BgaLayer(ch_vec.parse_next(input)?),
        BGA_LAYER2 => BgaLayer2(ch_vec.parse_next(input)?),
        BGA_ALPHA => BgaAlpha(hex_vec.parse_next(input)?),
        BGA_POOR_ALPHA => BgaPoorAlpha(hex_vec.parse_next(input)?),
        BGA_LAYER_ALPHA => BgaLayerAlpha(hex_vec.parse_next(input)?),
        BGA_LAYER2_ALPHA => BgaLayer2Alpha(hex_vec.parse_next(input)?),
        BGA_ARGB => BgaArgb(ch_vec.parse_next(input)?),
        BGA_POOR_ARGB => BgaPoorArgb(ch_vec.parse_next(input)?),
        BGA_LAYER_ARGB => BgaLayerArgb(ch_vec.parse_next(input)?),
        BGA_LAYER2_ARGB => BgaLayer2Argb(ch_vec.parse_next(input)?),
        SWITCH_BGA => SwitchBga(ch_vec.parse_next(input)?),
        LENGTH => Length(float.parse_next(input)?),
        BGM => Bgm(ch_vec.parse_next(input)?),
        BPM => Bpm(hex_vec
            .parse_next(input)?
            .into_iter()
            .map(|n| if n == 0 { None } else { Some(n as f64) })
            .collect()),
        EX_BPM => ExBpm(ch_vec.parse_next(input)?),
        NOTE_S..=NOTE_E => Note(ch, ch_vec.parse_next(input)?),
        INVISIBLE_NOTE_S..=INVISIBLE_NOTE_E => {
            InvisibleNote(ch, ch_vec.parse_next(input)?)
        }
        LONG_NOTE_S..=LONG_NOTE_E => LongNote(ch, ch_vec.parse_next(input)?),
        TEXT => Text(ch_vec.parse_next(input)?),
        LANDMINE_S..=LANDMINE_E => Landmine(
            ch,
            repeat(
                0..,
                preceded(space0, channel.map(|ch| ch.to_base_36() as f64)),
            )
            .parse_next(input)?,
        ),
        SCROLL => Scroll(ch_vec.parse_next(input)?),
        SPEED => Speed(ch_vec.parse_next(input)?),
        _ => Other(ch, rest_string.parse_next(input)?)
    };
    Ok(Token::Command(MainData(n, data)))
}
fn player(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (
        Caseless("PLAYER"),
        space1,
        dec_int.verify(|n| (1..=4).contains(n)),
    )
        .parse_next(input)?;
    Ok(Token::Command(Player(n)))
}
fn rank(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("RANK"), space1, dec_int).parse_next(input)?;
    Ok(Token::Command(Rank(n)))
}
fn def_ex_rank(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("DEFEXRANK"), space1, float).parse_next(input)?;
    Ok(Token::Command(DefExRank(n)))
}
fn ex_rank(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, n) =
        (Caseless("EXRANK"), channel, space1, float).parse_next(input)?;
    Ok(Token::Command(ExRank(ch, n)))
}
fn total(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("TOTAL"), space1, float).parse_next(input)?;
    Ok(Token::Command(Total(n)))
}

fn volume_wav(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("VOLWAV"), space1, float).parse_next(input)?;
    Ok(Token::Command(VolumeWav(n)))
}
fn stage_file(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("STAGEFILE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(StageFile(s)))
}
fn banner(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("BANNER"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Banner(s)))
}
fn back_bmp(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("BACKBMP"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(BackBmp(s)))
}
fn character_file(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("CHARFILE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(CharacterFile(s)))
}
fn play_level(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) =
        (Caseless("PLAYLEVEL"), space1, dec_int).parse_next(input)?;
    Ok(Token::Command(PlayLevel(n)))
}
fn difficulty(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) =
        (Caseless("DIFFICULTY"), space1, dec_int).parse_next(input)?;
    Ok(Token::Command(Difficulty(n)))
}
fn title(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) = (Caseless("TITLE"), one_of_space, quoted_or_no_quote)
        .parse_next(input)?;
    Ok(Token::Command(Title(s)))
}
fn sub_title(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("SUBTITLE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(SubTitle(s)))
}
fn artist(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("ARTIST"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Artist(s)))
}
fn sub_artist(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("SUBARTIST"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(SubArtist(s)))
}
fn maker(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("MAKER"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Maker(s)))
}
fn genre(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("GENRE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Genre(s)))
}
fn comment(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) = (Caseless("COMMENT"), one_of_space, quoted_or_no_quote)
        .parse_next(input)?;
    Ok(Token::Command(Comment(s)))
}
fn text(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, s) = (Caseless("TEXT"), channel, space1, escaped_string)
        .parse_next(input)?;
    Ok(Token::Command(Text(ch, s)))
}
fn song(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, s) = (Caseless("SONG"), channel, space1, escaped_string)
        .parse_next(input)?;
    Ok(Token::Command(Text(ch, s)))
}
fn path_wav(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("PATH_WAV"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(PathWav(s)))
}
fn bpm(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("BPM"), space1, float).parse_next(input)?;
    Ok(Token::Command(Bpm(n)))
}
fn ex_bpm(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, n) = (
        alt((Caseless("BPM"), Caseless("EXBPM"))),
        channel,
        space1,
        float,
    )
        .parse_next(input)?;
    Ok(Token::Command(ExBpm(ch, n)))
}
fn base_bpm(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("BASEBPM"), space1, float).parse_next(input)?;
    Ok(Token::Command(BaseBpm(n)))
}
fn stop(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, n) =
        (Caseless("STOP"), channel, space1, float).parse_next(input)?;
    Ok(Token::Command(Stop(ch, n)))
}
fn stp(input: &mut &str) -> ModalResult<Token> {
    let (_, _, x, _, y, _, z) = (
        Caseless("STP"),
        space1,
        padded_uint,
        ".",
        padded_uint.verify(|&n| n < 1000),
        space1,
        float,
    )
        .parse_next(input)?;
    Ok(Token::Command(Stp(x, y, z)))
}
fn ln_mode(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (
        Caseless("LNMODE"),
        space1,
        dec_int.verify(|n| (1..=3).contains(n)),
    )
        .parse_next(input)?;
    Ok(Token::Command(LnMode(n)))
}
fn ln_type(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (
        Caseless("LNTYPE"),
        space1,
        dec_int.verify(|n| (1..=2).contains(n)),
    )
        .parse_next(input)?;
    Ok(Token::Command(LnType(n)))
}
fn ln_object(input: &mut &str) -> ModalResult<Token> {
    let (_, _, ch) = (Caseless("LNOBJ"), space1, channel).parse_next(input)?;
    Ok(Token::Command(LnObject(ch)))
}
fn oct_fp(input: &mut &str) -> ModalResult<Token> {
    let _ = (Caseless("OCT/FP")).parse_next(input)?;
    Ok(Token::Command(OctFp))
}
fn option(input: &mut &str) -> ModalResult<Token> {
    let (_, _, name, _, option) = (
        Caseless("OPTION"),
        space1,
        take_while(0.., |c: char| c != ':'),
        ":",
        rest_string,
    )
        .parse_next(input)?;
    Ok(Token::Command(Option(name.to_string(), option)))
}
fn change_option(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, name, _, option) = (
        Caseless("CHANGEOPTION"),
        channel,
        space1,
        take_while(0.., |c: char| c != ':'),
        ":",
        rest_string,
    )
        .parse_next(input)?;
    Ok(Token::Command(ChangeOption(ch, name.to_string(), option)))
}
fn wav(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, s) =
        (Caseless("WAV"), channel, space1, rest_string).parse_next(input)?;
    Ok(Token::Command(Wav(ch, s)))
}
fn wav_command(input: &mut &str) -> ModalResult<Token> {
    let (_, _, id, _, ch, _, val) = (
        Caseless("WAVCMD"),
        space1,
        padded_uint.verify(|&id: &i32| id <= 2),
        space1,
        channel,
        space1,
        float,
    )
        .parse_next(input)?;
    Ok(Token::Command(WavCommand(id, ch, val)))
}
fn ex_wav(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, opt_str) =
        (Caseless("EXWAV"), channel, space1, alphanumeric1)
            .parse_next(input)?;
    let mut option = [None; 3];
    for c in opt_str.chars() {
        match c {
            'p' | 'P' => {
                option[0] = Some(
                    preceded(
                        space1,
                        float.verify(|p| (-10000.0..=1000.0).contains(p)),
                    )
                    .parse_next(input)?,
                );
            }
            'v' | 'V' => {
                option[1] = Some(
                    preceded(
                        space1,
                        float.verify(|v| (-10000.0..=0.0).contains(v)),
                    )
                    .parse_next(input)?,
                );
            }
            'f' | 'F' => {
                option[2] = Some(
                    preceded(
                        space1,
                        float.verify(|f| (100.0..=10000.0).contains(f)),
                    )
                    .parse_next(input)?,
                );
            }
            _ => {
                return Err(ParserError::from_input(input));
            }
        }
    }
    let name = preceded(space1, rest_string).parse_next(input)?;
    Ok(Token::Command(ExWav(ch, option, name)))
}
fn cdda(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("CDDA"), space1, dec_uint).parse_next(input)?;
    Ok(Token::Command(Cdda(n)))
}
fn midi_file(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("MIDIFILE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(MidiFile(s)))
}
fn bmp(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, s) =
        (Caseless("BMP"), channel, space1, rest_string).parse_next(input)?;
    Ok(Token::Command(Bmp(ch, s)))
}
fn ex_bmp(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, color, _, s): (_, _, _, Vec<u8>, _, _) = (
        Caseless("EXBMP"),
        channel,
        space1,
        separated(4, dec_uint.map(|n: u8| n), (space0, ",", space0)),
        one_of_space,
        rest_string,
    )
        .parse_next(input)?;
    Ok(Token::Command(ExBmp(ch, color.try_into().unwrap(), s)))
}
fn bga(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, ch_bmp) =
        (Caseless("BGA"), channel, space1, channel).parse_next(input)?;
    let p: Vec<f64> =
        repeat(6, preceded(space1, float.map(|n: f64| n))).parse_next(input)?;
    Ok(Token::Command(Bga(
        ch,
        ch_bmp,
        [[p[0], p[1]], [p[2], p[3]], [p[4], p[5]]],
    )))
}
fn at_bga(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, ch_bmp) =
        (Caseless("@BGA"), channel, space1, channel).parse_next(input)?;
    let p: Vec<f64> =
        repeat(6, preceded(space1, float.map(|n: f64| n))).parse_next(input)?;
    Ok(Token::Command(AtBga(
        ch,
        ch_bmp,
        [[p[0], p[1]], [p[2], p[3]], [p[4], p[5]]],
    )))
}
fn poor_bga(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (
        Caseless("POORBGA"),
        space1,
        dec_int.verify(|n| (0..=2).contains(n)),
    )
        .parse_next(input)?;
    Ok(Token::Command(PoorBga(n)))
}
fn switch_bga(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, frame, _, time, _, line, _, r#loop, _, argb, _, pattern) = (
        Caseless("SWBGA"),
        channel,
        space1,
        float,
        (space0, ":", space0),
        float,
        (space0, ":", space0),
        channel,
        (space0, ":", space0),
        dec_uint.map(|n: u32| n != 0),
        (space0, ":", space0),
        separated(4, dec_uint.map(|n: u8| n), (space0, ",", space0))
            .map(|v: Vec<u8>| v.try_into().unwrap()),
        space1,
        repeat(1.., channel),
    )
        .parse_next(input)?;
    Ok(Token::Command(SwitchBga(
        ch, frame, time, line, r#loop, argb, pattern,
    )))
}
fn argb(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, argb) = (
        Caseless("ARGB"),
        channel,
        space1,
        separated(4, dec_uint.map(|n: u8| n), (space0, ",", space0))
            .map(|v: Vec<u8>| v.try_into().unwrap()),
    )
        .parse_next(input)?;
    Ok(Token::Command(Argb(ch, argb)))
}
fn video_file(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("VIDEOFILE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(VideoFile(s)))
}
fn video_fps(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("VIDEOf/s"), space1, float).parse_next(input)?;
    Ok(Token::Command(VideoFps(n)))
}
fn video_colors(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) =
        (Caseless("VIDEOCOLORS"), space1, dec_uint).parse_next(input)?;
    Ok(Token::Command(VideoColors(n)))
}
fn video_delay(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) =
        (Caseless("VIDEODELAY"), space1, dec_uint).parse_next(input)?;
    Ok(Token::Command(VideoDelay(n)))
}
fn movie(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("MOVIE"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Movie(s)))
}
fn seek(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, n) =
        (Caseless("SEEK"), channel, space1, float).parse_next(input)?;
    Ok(Token::Command(Seek(ch, n)))
}
fn ex_character(input: &mut &str) -> ModalResult<Token> {
    let (_, _, spri_n, _, bmp_n, _, trim) = (
        Caseless("ExtChr"),
        space1,
        dec_uint.verify(|&n| n < 1024),
        space1,
        dec_uint.verify(|&n| n < 256),
        space1,
        separated(4, float.map(|f: f64| f), space1).map(|v: Vec<f64>| v),
    )
        .parse_next(input)?;
    let offset = opt(repeat(2, preceded(space1, float.map(|f: f64| f)))
        .map(|v: Vec<f64>| v.try_into().unwrap()))
    .parse_next(input)?;
    let abs = opt(repeat(2, preceded(space1, float.map(|f: f64| f)))
        .map(|v: Vec<f64>| v.try_into().unwrap()))
    .parse_next(input)?;
    Ok(Token::Command(ExCharacter(
        spri_n,
        bmp_n,
        [[trim[0], trim[1]], [trim[2], trim[3]]],
        offset,
        abs,
    )))
}
fn url(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("URL"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Url(s)))
}
fn email(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("EMAIL"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Email(s)))
}
fn scroll(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, n) =
        (Caseless("SCROLL"), channel, space1, float).parse_next(input)?;
    Ok(Token::Command(Scroll(ch, n)))
}
fn speed(input: &mut &str) -> ModalResult<Token> {
    let (_, ch, _, n) =
        (Caseless("SPEED"), channel, space1, float).parse_next(input)?;
    Ok(Token::Command(Speed(ch, n)))
}
fn preview(input: &mut &str) -> ModalResult<Token> {
    let (_, _, s) =
        (Caseless("PREVIEW"), one_of_space, rest_string).parse_next(input)?;
    Ok(Token::Command(Preview(s)))
}
fn base62(input: &mut &str) -> ModalResult<Token> {
    let (_, _, _): (_, _, i32) =
        (Caseless("BASE"), space1, dec_int.verify(|&n| n == 62))
            .parse_next(input)?;
    Ok(Token::Command(Base62))
}
fn random(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("RANDOM"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(Random(n)))
}
fn set_random(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) =
        (Caseless("SETRANDOM"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(SetRandom(n)))
}
fn end_random(input: &mut &str) -> ModalResult<Token> {
    let _ = Caseless("ENDRANDOM").parse_next(input)?;
    Ok(Token::ControlFlow(EndRandom))
}
fn r#if(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("IF"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(If(n)))
}
fn else_if(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("ELSEIF"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(ElseIf(n)))
}
fn r#else(input: &mut &str) -> ModalResult<Token> {
    let _ = Caseless("ELSE").parse_next(input)?;
    Ok(Token::ControlFlow(Else))
}
fn end_if(input: &mut &str) -> ModalResult<Token> {
    let _ = Caseless("ENDIF").parse_next(input)?;
    Ok(Token::ControlFlow(EndIf))
}
fn switch(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("SWITCH"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(Switch(n)))
}
fn set_switch(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) =
        (Caseless("SETSWITCH"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(SetSwitch(n)))
}
fn end_switch(input: &mut &str) -> ModalResult<Token> {
    let _ = Caseless("ENDSW").parse_next(input)?;
    Ok(Token::ControlFlow(EndSwitch))
}
fn case(input: &mut &str) -> ModalResult<Token> {
    let (_, _, n) = (Caseless("CASE"), space1, dec_uint).parse_next(input)?;
    Ok(Token::ControlFlow(Case(n)))
}
fn skip(input: &mut &str) -> ModalResult<Token> {
    let _ = Caseless("SKIP").parse_next(input)?;
    Ok(Token::ControlFlow(Skip))
}
fn default(input: &mut &str) -> ModalResult<Token> {
    let _ = Caseless("DEFAULT").parse_next(input)?;
    Ok(Token::ControlFlow(Default))
}
fn other(input: &mut &str) -> ModalResult<Token> {
    let (command, _, value) = (
        take_while(0.., |c: char| !c.is_whitespace()),
        space0,
        repeat(0.., any),
    )
        .parse_next(input)?;
    Ok(Token::Command(Other(command.to_string(), value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn space_test() {
        // one_of_space
        assert_eq!(one_of_space.parse_peek("  test"), Ok((" test", ' ')));
        assert_eq!(one_of_space.parse_peek("　　test"), Ok(("　test", '　')));
        assert_eq!(one_of_space.parse_peek("\t\ttest"), Ok(("\ttest", '\t')));
        assert_eq!(one_of_space.parse_peek("\n\ntest"), Ok(("\ntest", '\n')));
        assert!(one_of_space.parse_peek("てすと").is_err());

        // space0
        assert_eq!(space0.parse_peek("  test"), Ok(("test", "  ")));
        assert_eq!(space0.parse_peek("　　test"), Ok(("test", "　　")));
        assert_eq!(space0.parse_peek("\t\ttest"), Ok(("test", "\t\t")));
        assert_eq!(space0.parse_peek("\n\ntest"), Ok(("test", "\n\n")));
        assert_eq!(space0.parse_peek("てすと"), Ok(("てすと", "")));

        // space1
        assert_eq!(space1.parse_peek("  test"), Ok(("test", "  ")));
        assert_eq!(space1.parse_peek("　　test"), Ok(("test", "　　")));
        assert_eq!(space1.parse_peek("\t\ttest"), Ok(("test", "\t\t")));
        assert_eq!(space1.parse_peek("\n\ntest"), Ok(("test", "\n\n")));
        assert!(space1.parse_peek("てすと").is_err());
    }

    #[test]
    fn channel_test() {
        assert_eq!(channel.parse_peek("00?"), Ok(("?", Channel::from("00"))));
        assert_eq!(channel.parse_peek("99?"), Ok(("?", Channel::from("99"))));
        assert_eq!(channel.parse_peek("FF?"), Ok(("?", Channel::from("FF"))));
        assert_eq!(channel.parse_peek("ZZ?"), Ok(("?", Channel::from("ZZ"))));
        assert_eq!(channel.parse_peek("ff?"), Ok(("?", Channel::from("ff"))));
        assert_eq!(channel.parse_peek("zz?"), Ok(("?", Channel::from("zz"))));
        assert!(channel.parse_peek("てすと").is_err());
    }

    #[test]
    fn quoted_or_no_quote_test() {
        assert_eq!(
            quoted_or_no_quote.parse_peek(r#""Test""#),
            Ok(("", String::from("Test")))
        );
        assert_eq!(
            quoted_or_no_quote.parse_peek(r#"Test"#),
            Ok(("", String::from("Test")))
        );
        assert_eq!(
            quoted_or_no_quote.parse_peek(r#""\n\r\t\\\"""#),
            Ok(("", String::from("\n\r\t\\\"")))
        );
        assert_eq!(
            quoted_or_no_quote.parse_peek(r#""\u{2014}""#),
            Ok(("", String::from("\u{2014}")))
        );
    }

    #[test]
    fn command_test() {
        // PLAYER
        assert_eq!(
            command.parse_peek("#PLAYER 1"),
            Ok(("", Token::Command(Player(1))))
        );
        assert_eq!(
            command.parse_peek("#player 2"),
            Ok(("", Token::Command(Player(2))))
        );
        assert_eq!(
            command.parse_peek("#PLAYER 0"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("PLAYER"),
                    String::from("0")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#Player 5"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("Player"),
                    String::from("5")
                ))
            ))
        );
        // RANK
        assert_eq!(
            command.parse_peek("#RANK 0"),
            Ok(("", Token::Command(Rank(0))))
        );
        assert_eq!(
            command.parse_peek("#rank 100"),
            Ok(("", Token::Command(Rank(100))))
        );
        assert_eq!(
            command.parse_peek("#Rank -100"),
            Ok(("", Token::Command(Rank(-100))))
        );
        // DEFEXRANK
        assert_eq!(
            command.parse_peek("#DEFEXRANK 0"),
            Ok(("", Token::Command(DefExRank(0.))))
        );
        assert_eq!(
            command.parse_peek("#defexrank 100.5"),
            Ok(("", Token::Command(DefExRank(100.5))))
        );
        assert_eq!(
            command.parse_peek("#DefExRank -123e-7"),
            Ok(("", Token::Command(DefExRank(-123e-7))))
        );
        // EXRANK
        assert_eq!(
            command.parse_peek("#EXRANK01 0"),
            Ok(("", Token::Command(ExRank(Channel::from("01"), 0.))))
        );
        assert_eq!(
            command.parse_peek("#exrankZZ 100.5"),
            Ok(("", Token::Command(ExRank(Channel::from("ZZ"), 100.5))))
        );
        assert_eq!(
            command.parse_peek("#ExRankff -123E-7"),
            Ok(("", Token::Command(ExRank(Channel::from("ff"), -123E-7))))
        );
        // TOTAL
        assert_eq!(
            command.parse_peek("#TOTAL 250"),
            Ok(("", Token::Command(Total(250.))))
        );
        assert_eq!(
            command.parse_peek("#total 12345.6789e123"),
            Ok(("", Token::Command(Total(12345.6789e123))))
        );
        // VOLWAV
        assert_eq!(
            command.parse_peek("#VOLWAV 100"),
            Ok(("", Token::Command(VolumeWav(100.))))
        );
        assert_eq!(
            command.parse_peek("#volwav 123.4"),
            Ok(("", Token::Command(VolumeWav(123.4))))
        );
        // STAGEFILE
        assert_eq!(
            command.parse_peek("#STAGEFILE image.bmp"),
            Ok(("", Token::Command(StageFile(String::from("image.bmp")))))
        );
        assert_eq!(
            command.parse_peek("#stagefile 画像.png"),
            Ok(("", Token::Command(StageFile(String::from("画像.png")))))
        );
        // BANNER
        assert_eq!(
            command.parse_peek("#BANNER banner.jpg"),
            Ok(("", Token::Command(Banner(String::from("banner.jpg")))))
        );
        assert_eq!(
            command.parse_peek("#banner ばなー.bmp"),
            Ok(("", Token::Command(Banner(String::from("ばなー.bmp")))))
        );
        // BACKBMP
        assert_eq!(
            command.parse_peek("#BACKBMP back.png"),
            Ok(("", Token::Command(BackBmp(String::from("back.png")))))
        );
        assert_eq!(
            command.parse_peek("#backbmp 背景.jpg"),
            Ok(("", Token::Command(BackBmp(String::from("背景.jpg")))))
        );
        // CHARFILE
        assert_eq!(
            command.parse_peek("#CHARFILE character.chp"),
            Ok((
                "",
                Token::Command(CharacterFile(String::from("character.chp")))
            ))
        );
        assert_eq!(
            command.parse_peek("#charfile キャラファイル.chp"),
            Ok((
                "",
                Token::Command(CharacterFile(String::from(
                    "キャラファイル.chp"
                )))
            ))
        );
        // PLAYLEVEL
        assert_eq!(
            command.parse_peek("#PLAYLEVEL 12"),
            Ok(("", Token::Command(PlayLevel(12))))
        );
        assert_eq!(
            command.parse_peek("#playlevel 999"),
            Ok(("", Token::Command(PlayLevel(999))))
        );
        // DIFFICULTY
        assert_eq!(
            command.parse_peek("#DIFFICULTY 1"),
            Ok(("", Token::Command(Difficulty(1))))
        );
        assert_eq!(
            command.parse_peek("#difficulty 5"),
            Ok(("", Token::Command(Difficulty(5))))
        );
        // TITLE
        assert_eq!(
            command.parse_peek("#TITLE \"title\""),
            Ok(("", Token::Command(Title(String::from("title")))))
        );
        assert_eq!(
            command.parse_peek("#title タイトル"),
            Ok(("", Token::Command(Title(String::from("タイトル")))))
        );
        assert_eq!(
            command.parse_peek("#Title  　ABC　 "),
            Ok(("", Token::Command(Title(String::from(" 　ABC　 ")))))
        );
        // SUBTITLE
        assert_eq!(
            command.parse_peek("#SUBTITLE sub_title"),
            Ok(("", Token::Command(SubTitle(String::from("sub_title")))))
        );
        assert_eq!(
            command.parse_peek("#subtitle サブタイトル"),
            Ok(("", Token::Command(SubTitle(String::from("サブタイトル")))))
        );
        assert_eq!(
            command.parse_peek("#SubTitle \tLOVE♡SHINE\t"),
            Ok(("", Token::Command(SubTitle(String::from("\tLOVE♡SHINE\t")))))
        );
        // ARTIST
        assert_eq!(
            command.parse_peek("#ARTIST artist"),
            Ok(("", Token::Command(Artist(String::from("artist")))))
        );
        assert_eq!(
            command.parse_peek("#artist アーティスト"),
            Ok(("", Token::Command(Artist(String::from("アーティスト")))))
        );
        // SUBARTIST
        assert_eq!(
            command.parse_peek("#SUBARTIST sub_artist"),
            Ok(("", Token::Command(SubArtist(String::from("sub_artist")))))
        );
        assert_eq!(
            command.parse_peek("#subartist サブアーティスト"),
            Ok((
                "",
                Token::Command(SubArtist(String::from("サブアーティスト")))
            ))
        );
        // MAKER
        assert_eq!(
            command.parse_peek("#MAKER maker"),
            Ok(("", Token::Command(Maker(String::from("maker")))))
        );
        assert_eq!(
            command.parse_peek("#maker 譜面制作者"),
            Ok(("", Token::Command(Maker(String::from("譜面制作者")))))
        );
        // GENRE
        assert_eq!(
            command.parse_peek("#GENRE genre"),
            Ok(("", Token::Command(Genre(String::from("genre")))))
        );
        assert_eq!(
            command.parse_peek("#genre ジャンル"),
            Ok(("", Token::Command(Genre(String::from("ジャンル")))))
        );
        // COMMENT
        assert_eq!(
            command.parse_peek("#COMMENT \"comment\""),
            Ok(("", Token::Command(Comment(String::from("comment")))))
        );
        assert_eq!(
            command.parse_peek("#comment コメント"),
            Ok(("", Token::Command(Comment(String::from("コメント")))))
        );
        assert_eq!(
            command.parse_peek("#Comment \"𠮷野家\""),
            Ok(("", Token::Command(Comment(String::from("𠮷野家")))))
        );
        // TEXT
        assert_eq!(
            command.parse_peek("#TEXT01 \"歌詞\""),
            Ok((
                "",
                Token::Command(Text(Channel::from("01"), String::from("歌詞")))
            ))
        );
        assert_eq!(
            command.parse_peek("#textzz \"瑕疵\""),
            Ok((
                "",
                Token::Command(Text(Channel::from("zz"), String::from("瑕疵")))
            ))
        );
        // SONG
        assert_eq!(
            command.parse_peek("#SONG01 \"歌詞\""),
            Ok((
                "",
                Token::Command(Text(Channel::from("01"), String::from("歌詞")))
            ))
        );
        assert_eq!(
            command.parse_peek("#songzz \"瑕疵\""),
            Ok((
                "",
                Token::Command(Text(Channel::from("zz"), String::from("瑕疵")))
            ))
        );
        // PATH_WAV
        assert_eq!(
            command.parse_peek("#PATH_WAV C:/path/to/wav"),
            Ok(("", Token::Command(PathWav(String::from("C:/path/to/wav")))))
        );
        assert_eq!(
            command.parse_peek("#path_wav local/path"),
            Ok(("", Token::Command(PathWav(String::from("local/path")))))
        );
        // BPM
        assert_eq!(
            command.parse_peek("#BPM 120"),
            Ok(("", Token::Command(Bpm(120.))))
        );
        assert_eq!(
            command.parse_peek("#BPM 222.22"),
            Ok(("", Token::Command(Bpm(222.22))))
        );
        // BPM EXBPM
        assert_eq!(
            command.parse_peek("#BPM01 1.2e-10"),
            Ok(("", Token::Command(ExBpm(Channel::from("01"), 1.2e-10))))
        );
        assert_eq!(
            command.parse_peek("#EXBPM01 1.2e-10"),
            Ok(("", Token::Command(ExBpm(Channel::from("01"), 1.2e-10))))
        );
        assert_eq!(
            command.parse_peek("#bpmzz 123456789"),
            Ok(("", Token::Command(ExBpm(Channel::from("zz"), 123456789.))))
        );
        assert_eq!(
            command.parse_peek("#exbpmzz 123456789."),
            Ok(("", Token::Command(ExBpm(Channel::from("zz"), 123456789.))))
        );
        // BASEBPM
        assert_eq!(
            command.parse_peek("#BASEBPM 80."),
            Ok(("", Token::Command(BaseBpm(80.))))
        );
        assert_eq!(
            command.parse_peek("#basebpm 800."),
            Ok(("", Token::Command(BaseBpm(800.))))
        );
        // STOP
        assert_eq!(
            command.parse_peek("#STOP01 192"),
            Ok(("", Token::Command(Stop(Channel::from("01"), 192.))))
        );
        assert_eq!(
            command.parse_peek("#stopzz 1e20"),
            Ok(("", Token::Command(Stop(Channel::from("zz"), 1e20))))
        );
        // STP
        assert_eq!(
            command.parse_peek("#STP 000.000 1000"),
            Ok(("", Token::Command(Stp(0, 0, 1000.))))
        );
        assert_eq!(
            command.parse_peek("#stp 999.999 0.1"),
            Ok(("", Token::Command(Stp(999, 999, 0.1))))
        );
        assert_eq!(
            command.parse_peek("#stp 500.1000 500"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("stp"),
                    String::from("500.1000 500")
                ))
            ))
        );
        // LNTYPE
        assert_eq!(
            command.parse_peek("#LNTYPE 1"),
            Ok(("", Token::Command(LnType(1))))
        );
        assert_eq!(
            command.parse_peek("#lntype 2"),
            Ok(("", Token::Command(LnType(2))))
        );
        assert_eq!(
            command.parse_peek("#LnType 0"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("LnType"),
                    String::from("0")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#LNTYPE 3"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("LNTYPE"),
                    String::from("3")
                ))
            ))
        );
        // LNOBJ
        assert_eq!(
            command.parse_peek("#LNOBJ 01"),
            Ok(("", Token::Command(LnObject(Channel::from("01")))))
        );
        assert_eq!(
            command.parse_peek("#lnobj zz"),
            Ok(("", Token::Command(LnObject(Channel::from("zz")))))
        );
        // OCT/FP
        assert_eq!(
            command.parse_peek("#OCT/FP"),
            Ok(("", Token::Command(OctFp)))
        );
        assert_eq!(
            command.parse_peek("#oct/fp"),
            Ok(("", Token::Command(OctFp)))
        );
        // OPTION
        assert_eq!(
            command.parse_peek("#option GameName:OptionStr"),
            Ok((
                "",
                Token::Command(Option(
                    String::from("GameName"),
                    String::from("OptionStr")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#OPTION 774:HI-SPEED_x99.75"),
            Ok((
                "",
                Token::Command(Option(
                    String::from("774"),
                    String::from("HI-SPEED_x99.75")
                ))
            ))
        );
        // CHANGEOPTION
        assert_eq!(
            command.parse_peek("#CHANGEOPTION01 charatbeatHDX:LONGMODE 0"),
            Ok((
                "",
                Token::Command(ChangeOption(
                    Channel::from("01"),
                    String::from("charatbeatHDX"),
                    String::from("LONGMODE 0")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#changeoptionzz 774:RANDOM_MIRROR"),
            Ok((
                "",
                Token::Command(ChangeOption(
                    Channel::from("zz"),
                    String::from("774"),
                    String::from("RANDOM_MIRROR")
                ))
            ))
        );
        // WAV
        assert_eq!(
            command.parse_peek("#WAV01 base.wav"),
            Ok((
                "",
                Token::Command(Wav(
                    Channel::from("01"),
                    String::from("base.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#WAVzz kick.ogg"),
            Ok((
                "",
                Token::Command(Wav(
                    Channel::from("zz"),
                    String::from("kick.ogg")
                ))
            ))
        );
        // WAVCMD
        assert_eq!(
            command.parse_peek("#WAVCMD 00 01 60"),
            Ok(("", Token::Command(WavCommand(0, Channel::from("01"), 60.))))
        );
        assert_eq!(
            command.parse_peek("#wavcmd 02 zz 2000."),
            Ok((
                "",
                Token::Command(WavCommand(2, Channel::from("zz"), 2000.))
            ))
        );
        assert_eq!(
            command.parse_peek("#WavCmd 03 zz 2000."),
            Ok((
                "",
                Token::Command(Other(
                    String::from("WavCmd"),
                    String::from("03 zz 2000.")
                ))
            ))
        );
        // EXWAV
        assert_eq!(
            command.parse_peek("#EXWAV01 vfp -50 100 -10000 aaa.wav"),
            Ok((
                "",
                Token::Command(ExWav(
                    Channel::from("01"),
                    [Some(-10000.), Some(-50.), Some(100.)],
                    String::from("aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#exwavzz   P            500 aaa.wav"),
            Ok((
                "",
                Token::Command(ExWav(
                    Channel::from("zz"),
                    [Some(500.), None, None],
                    String::from("aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExWavFF p 10000.001 aaa.wav"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExWavFF"),
                    String::from("p 10000.001 aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExWavFF p -10000.001 aaa.wav"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExWavFF"),
                    String::from("p -10000.001 aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExWavFF v 0.001 aaa.wav"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExWavFF"),
                    String::from("v 0.001 aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExWavFF v -10000.001 aaa.wav"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExWavFF"),
                    String::from("v -10000.001 aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExWavFF f 100000.001 aaa.wav"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExWavFF"),
                    String::from("f 100000.001 aaa.wav")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExWavFF f 99.999 aaa.wav"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExWavFF"),
                    String::from("f 99.999 aaa.wav")
                ))
            ))
        );
        // CDDA
        assert_eq!(
            command.parse_peek("#CDDA 0"),
            Ok(("", Token::Command(Cdda(0))))
        );
        assert_eq!(
            command.parse_peek("#cdda 5"),
            Ok(("", Token::Command(Cdda(5))))
        );
        // MIDIFILE
        assert_eq!(
            command.parse_peek("#MIDIFILE piano.mid"),
            Ok(("", Token::Command(MidiFile(String::from("piano.mid")))))
        );
        assert_eq!(
            command.parse_peek("#midifile base.mid"),
            Ok(("", Token::Command(MidiFile(String::from("base.mid")))))
        );
        // BMP
        assert_eq!(
            command.parse_peek("#BMP00 miss.bmp"),
            Ok((
                "",
                Token::Command(Bmp(
                    Channel::from("00"),
                    String::from("miss.bmp")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#bmpzz bga.mp4"),
            Ok((
                "",
                Token::Command(Bmp(
                    Channel::from("zz"),
                    String::from("bga.mp4")
                ))
            ))
        );
        // EXBMP
        assert_eq!(
            command.parse_peek("#EXBMP00 0,0,0,0 miss.avi"),
            Ok((
                "",
                Token::Command(ExBmp(
                    Channel::from("00"),
                    [0; 4],
                    String::from("miss.avi")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#exbmpzz 255,255,255,255 bga.webm"),
            Ok((
                "",
                Token::Command(ExBmp(
                    Channel::from("zz"),
                    [255; 4],
                    String::from("bga.webm")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExBmpFF 256,0,0,0 movie.mov"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExBmpFF"),
                    String::from("256,0,0,0 movie.mov")
                ))
            ))
        );
        // BGA
        assert_eq!(
            command.parse_peek("#BGA00 00 64 64 128 128 0 0"),
            Ok((
                "",
                Token::Command(Bga(
                    Channel::from("00"),
                    Channel::from("00"),
                    [[64., 64.], [128., 128.], [0., 0.]]
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#bgazz ZZ 1.1 2.2 3.3 4.4 5.5 6.6"),
            Ok((
                "",
                Token::Command(Bga(
                    Channel::from("zz"),
                    Channel::from("ZZ"),
                    [[1.1, 2.2], [3.3, 4.4], [5.5, 6.6]]
                ))
            ))
        );
        // @BGA
        assert_eq!(
            command.parse_peek("#@BGA00 00 64 64 128 128 0 0"),
            Ok((
                "",
                Token::Command(AtBga(
                    Channel::from("00"),
                    Channel::from("00"),
                    [[64., 64.], [128., 128.], [0., 0.]]
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#@bgazz ZZ 1.1 2.2 3.3 4.4 5.5 6.6"),
            Ok((
                "",
                Token::Command(AtBga(
                    Channel::from("zz"),
                    Channel::from("ZZ"),
                    [[1.1, 2.2], [3.3, 4.4], [5.5, 6.6]]
                ))
            ))
        );
        // POORBGA
        assert_eq!(
            command.parse_peek("#POORBGA 0"),
            Ok(("", Token::Command(PoorBga(0))))
        );
        assert_eq!(
            command.parse_peek("#poorbga 2"),
            Ok(("", Token::Command(PoorBga(2))))
        );
        assert_eq!(
            command.parse_peek("#PoorBga 3"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("PoorBga"),
                    String::from("3")
                ))
            ))
        );
        // SWBGA
        assert_eq!(
            command.parse_peek("#SWBGA01 0:0:11:0:0,0,0,0 00"),
            Ok((
                "",
                Token::Command(SwitchBga(
                    Channel::from("01"),
                    0.,
                    0.,
                    Channel::from("11"),
                    false,
                    [0; 4],
                    vec![Channel::from("00"),]
                ))
            ))
        );
        assert_eq!(
            command
                .parse_peek("#SWBGA01 100:400:16:0:255,255,255,255 01020304"),
            Ok((
                "",
                Token::Command(SwitchBga(
                    Channel::from("01"),
                    100.,
                    400.,
                    Channel::from("16"),
                    false,
                    [255; 4],
                    vec![
                        Channel::from("01"),
                        Channel::from("02"),
                        Channel::from("03"),
                        Channel::from("04"),
                    ]
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#SWBGA01 100:400:16:0:255,255,255,256 01"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("SWBGA01"),
                    String::from("100:400:16:0:255,255,255,256 01")
                ))
            ))
        );
        // ARGB
        assert_eq!(
            command.parse_peek("#ARGB01 0,0,0,0"),
            Ok(("", Token::Command(Argb(Channel::from("01"), [0; 4]))))
        );
        assert_eq!(
            command.parse_peek("#argbzz 255,255,255,255"),
            Ok(("", Token::Command(Argb(Channel::from("zz"), [255; 4]))))
        );
        assert_eq!(
            command.parse_peek("#ArgbFF 255,255,255,256"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ArgbFF"),
                    String::from("255,255,255,256")
                ))
            ))
        );
        // VIDEOFILE
        assert_eq!(
            command.parse_peek("#VIDEOFILE video.mp4"),
            Ok(("", Token::Command(VideoFile(String::from("video.mp4")))))
        );
        assert_eq!(
            command.parse_peek("#videofile bga.avi"),
            Ok(("", Token::Command(VideoFile(String::from("bga.avi")))))
        );
        // VIDEOf/p
        assert_eq!(
            command.parse_peek("#VIDEOf/s 60"),
            Ok(("", Token::Command(VideoFps(60.))))
        );
        assert_eq!(
            command.parse_peek("#videoF/S 59.94"),
            Ok(("", Token::Command(VideoFps(59.94))))
        );
        // VIDEOCOLORS
        assert_eq!(
            command.parse_peek("#VIDEOCOLORS 16"),
            Ok(("", Token::Command(VideoColors(16))))
        );
        assert_eq!(
            command.parse_peek("#videocolors 32"),
            Ok(("", Token::Command(VideoColors(32))))
        );
        // VIDEODELAY
        assert_eq!(
            command.parse_peek("#VIDEODELAY 60"),
            Ok(("", Token::Command(VideoDelay(60))))
        );
        assert_eq!(
            command.parse_peek("#videodelay 1234"),
            Ok(("", Token::Command(VideoDelay(1234))))
        );
        // MOVIE
        assert_eq!(
            command.parse_peek("#MOVIE movie.mp4"),
            Ok(("", Token::Command(Movie(String::from("movie.mp4")))))
        );
        assert_eq!(
            command.parse_peek("#movie bga.avi"),
            Ok(("", Token::Command(Movie(String::from("bga.avi")))))
        );
        // SEEK
        assert_eq!(
            command.parse_peek("#SEEK01 1000"),
            Ok(("", Token::Command(Seek(Channel::from("01"), 1000.))))
        );
        assert_eq!(
            command.parse_peek("#seekzz 1234"),
            Ok(("", Token::Command(Seek(Channel::from("zz"), 1234.))))
        );
        // ExtChr
        assert_eq!(
            command.parse_peek("#ExtChr 0 0 0 0 0 0"),
            Ok((
                "",
                Token::Command(ExCharacter(0, 0, [[0.; 2]; 2], None, None))
            ))
        );
        assert_eq!(
            command.parse_peek("#EXTCHR 1023 0 0 0 0 0 1 1"),
            Ok((
                "",
                Token::Command(ExCharacter(
                    1023,
                    0,
                    [[0.; 2]; 2],
                    Some([1.; 2]),
                    None
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#extchr 0 255 0 0 0 0 1 1 2 2"),
            Ok((
                "",
                Token::Command(ExCharacter(
                    0,
                    255,
                    [[0.; 2]; 2],
                    Some([1.; 2]),
                    Some([2.; 2])
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExtChr 1024 0 0 0 0 0"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExtChr"),
                    String::from("1024 0 0 0 0 0")
                ))
            ))
        );
        assert_eq!(
            command.parse_peek("#ExtChr 0 256 0 0 0 0"),
            Ok((
                "",
                Token::Command(Other(
                    String::from("ExtChr"),
                    String::from("0 256 0 0 0 0")
                ))
            ))
        );
        // URL
        assert_eq!(
            command.parse_peek("%URL https://home-page.net"),
            Ok((
                "",
                Token::Command(Url(String::from("https://home-page.net")))
            ))
        );
        assert_eq!(
            command.parse_peek("%url https://foo.com"),
            Ok(("", Token::Command(Url(String::from("https://foo.com")))))
        );
        // EMAIL
        assert_eq!(
            command.parse_peek("%EMAIL name@some.mail.com"),
            Ok((
                "",
                Token::Command(Email(String::from("name@some.mail.com")))
            ))
        );
        assert_eq!(
            command.parse_peek("%email foo@some.mail.co.jp"),
            Ok((
                "",
                Token::Command(Email(String::from("foo@some.mail.co.jp")))
            ))
        );
        // SCROLL
        assert_eq!(
            command.parse_peek("#SCROLL01 1"),
            Ok(("", Token::Command(Scroll(Channel::from("01"), 1.))))
        );
        assert_eq!(
            command.parse_peek("#scrollzz 0.5"),
            Ok(("", Token::Command(Scroll(Channel::from("zz"), 0.5))))
        );
        // SPEED
        assert_eq!(
            command.parse_peek("#SPEED01 1"),
            Ok(("", Token::Command(Speed(Channel::from("01"), 1.))))
        );
        assert_eq!(
            command.parse_peek("#speedzz 0.5"),
            Ok(("", Token::Command(Speed(Channel::from("zz"), 0.5))))
        );
        // PREVIEW
        assert_eq!(
            command.parse_peek("#PREVIEW preview.wav"),
            Ok(("", Token::Command(Preview(String::from("preview.wav")))))
        );
        assert_eq!(
            command.parse_peek("#preview プレビュー.ogg"),
            Ok(("", Token::Command(Preview(String::from("プレビュー.ogg")))))
        );
        // BASE62
        assert_eq!(
            command.parse_peek("#BASE 62"),
            Ok(("", Token::Command(Base62)))
        );
        assert_eq!(
            command.parse_peek("#base 62"),
            Ok(("", Token::Command(Base62)))
        );
        // RANDOM
        assert_eq!(
            command.parse_peek("#RANDOM 1"),
            Ok(("", Token::ControlFlow(Random(1))))
        );
        assert_eq!(
            command.parse_peek("#random 123456789012345678901234567890"),
            Ok((
                "",
                Token::ControlFlow(Random(123456789012345678901234567890))
            ))
        );
        // SETRANDOM
        assert_eq!(
            command.parse_peek("#SETRANDOM 1"),
            Ok(("", Token::ControlFlow(SetRandom(1))))
        );
        assert_eq!(
            command.parse_peek("#setrandom 123456789012345678901234567890"),
            Ok((
                "",
                Token::ControlFlow(SetRandom(123456789012345678901234567890))
            ))
        );
        // ENDRANDOM
        assert_eq!(
            command.parse_peek("#ENDRANDOM"),
            Ok(("", Token::ControlFlow(EndRandom)))
        );
        assert_eq!(
            command.parse_peek("#endrandom"),
            Ok(("", Token::ControlFlow(EndRandom)))
        );
        // IF
        assert_eq!(
            command.parse_peek("#IF 1"),
            Ok(("", Token::ControlFlow(If(1))))
        );
        assert_eq!(
            command.parse_peek("#if 123456789012345678901234567890"),
            Ok(("", Token::ControlFlow(If(123456789012345678901234567890))))
        );
        // ELSEIF
        assert_eq!(
            command.parse_peek("#ELSEIF 1"),
            Ok(("", Token::ControlFlow(ElseIf(1))))
        );
        assert_eq!(
            command.parse_peek("#elseif 123456789012345678901234567890"),
            Ok((
                "",
                Token::ControlFlow(ElseIf(123456789012345678901234567890))
            ))
        );
        // ELSE
        assert_eq!(
            command.parse_peek("#ELSE"),
            Ok(("", Token::ControlFlow(Else)))
        );
        assert_eq!(
            command.parse_peek("#else"),
            Ok(("", Token::ControlFlow(Else)))
        );
        // ENDIF
        assert_eq!(
            command.parse_peek("#ENDIF"),
            Ok(("", Token::ControlFlow(EndIf)))
        );
        assert_eq!(
            command.parse_peek("#endif"),
            Ok(("", Token::ControlFlow(EndIf)))
        );
        // SWITCH
        assert_eq!(
            command.parse_peek("#SWITCH 1"),
            Ok(("", Token::ControlFlow(Switch(1))))
        );
        assert_eq!(
            command.parse_peek("#switch 123456789012345678901234567890"),
            Ok((
                "",
                Token::ControlFlow(Switch(123456789012345678901234567890))
            ))
        );
        // SETSWITCH
        assert_eq!(
            command.parse_peek("#SETSWITCH 1"),
            Ok(("", Token::ControlFlow(SetSwitch(1))))
        );
        assert_eq!(
            command.parse_peek("#setswitch 123456789012345678901234567890"),
            Ok((
                "",
                Token::ControlFlow(SetSwitch(123456789012345678901234567890))
            ))
        );
        // ENDSWITCH
        assert_eq!(
            command.parse_peek("#ENDSW"),
            Ok(("", Token::ControlFlow(EndSwitch)))
        );
        assert_eq!(
            command.parse_peek("#endsw"),
            Ok(("", Token::ControlFlow(EndSwitch)))
        );
        // CASE
        assert_eq!(
            command.parse_peek("#CASE 1"),
            Ok(("", Token::ControlFlow(Case(1))))
        );
        assert_eq!(
            command.parse_peek("#case 123456789012345678901234567890"),
            Ok(("", Token::ControlFlow(Case(123456789012345678901234567890))))
        );
        // SKIP
        assert_eq!(
            command.parse_peek("#SKIP"),
            Ok(("", Token::ControlFlow(Skip)))
        );
        assert_eq!(
            command.parse_peek("#skip"),
            Ok(("", Token::ControlFlow(Skip)))
        );
        // DEFAULT
        assert_eq!(
            command.parse_peek("#DEFAULT"),
            Ok(("", Token::ControlFlow(Default)))
        );
        assert_eq!(
            command.parse_peek("#default"),
            Ok(("", Token::ControlFlow(Default)))
        );
    }
}
