pub(crate) mod lex;
pub(crate) mod parse;
pub(crate) mod token;
pub use token::Channel;

/// ファイルを解析したままのBMS
///
/// ランダム要素を確定していない。
/// [`RawBms::make_bms`]で疑似乱数生成器を指定してBMSを生成する
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RawBms {
    raw_bms: BmsBlock,
    all_wav_files: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct BmsBlock(Vec<BmsElement>);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BmsElement {
    Command(token::Command),
    Random(BmsRandomBlock),
    Switch(BmsSwitchBlock),
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BmsRandomBlock(RandomValue, Vec<BmsRandomElement>);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BmsRandomElement {
    Block(BmsBlock),
    IfBlock(BmsIfBlock),
}
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BmsIfBlock {
    pub(crate) r#if: Vec<(u128, BmsBlock)>,
    pub(crate) r#else: Option<BmsBlock>,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BmsSwitchBlock(
    RandomValue,
    Vec<BmsCaseBlock>,
    std::collections::HashSet<u128>,
);
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BmsCaseBlock(SwitchLabel, BmsBlock, bool);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RandomValue {
    Max(u128),
    Set(u128),
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SwitchLabel {
    Case(u128),
    Default,
}
impl BmsBlock {
    pub(crate) fn get_token_vec<'a, Rng: rand::RngCore>(
        &'a self,
        output: &mut Vec<&'a token::Command>,
        rng: &mut Rng,
    ) {
        for e in &self.0 {
            e.get_token_vec(output, rng);
        }
    }
}
impl BmsElement {
    fn get_token_vec<'a, Rng: rand::RngCore>(
        &'a self,
        output: &mut Vec<&'a token::Command>,
        rng: &mut Rng,
    ) {
        match self {
            BmsElement::Command(c) => {
                output.push(c);
            }
            BmsElement::Random(rb) => {
                rb.get_token_vec(output, rng);
            }
            BmsElement::Switch(sb) => {
                sb.get_token_vec(output, rng);
            }
        }
    }
}
impl BmsRandomBlock {
    fn get_token_vec<'a, Rng: rand::RngCore>(
        &'a self,
        output: &mut Vec<&'a token::Command>,
        rng: &mut Rng,
    ) {
        use rand::Rng;
        let n = match self.0 {
            RandomValue::Max(n) => rng.random_range(1..=n),
            RandomValue::Set(n) => n,
        };
        for e in &self.1 {
            e.get_token_vec(output, rng, n);
        }
    }
}
impl BmsRandomElement {
    fn get_token_vec<'a, Rng: rand::RngCore>(
        &'a self,
        output: &mut Vec<&'a token::Command>,
        rng: &mut Rng,
        n: u128,
    ) {
        match self {
            BmsRandomElement::Block(b) => {
                b.get_token_vec(output, rng);
            }
            BmsRandomElement::IfBlock(ib) => {
                ib.get_token_vec(output, rng, n);
            }
        }
    }
}
impl BmsIfBlock {
    fn get_token_vec<'a, Rng: rand::RngCore>(
        &'a self,
        output: &mut Vec<&'a token::Command>,
        rng: &mut Rng,
        n: u128,
    ) {
        for (i, b) in &self.r#if {
            if *i == n {
                b.get_token_vec(output, rng);
                return;
            }
        }
        if let Some(b) = &self.r#else {
            b.get_token_vec(output, rng);
        }
    }
}
impl BmsSwitchBlock {
    fn get_token_vec<'a, Rng: rand::RngCore>(
        &'a self,
        output: &mut Vec<&'a token::Command>,
        rng: &mut Rng,
    ) {
        use rand::Rng;
        let n = match self.0 {
            RandomValue::Max(n) => rng.random_range(1..=n),
            RandomValue::Set(n) => n,
        };
        let mut flag = false;
        for e in &self.1 {
            if match &e.0 {
                SwitchLabel::Case(i) => *i == n,
                SwitchLabel::Default => !self.2.contains(&n),
            } {
                flag = true;
            }
            if flag {
                e.1.get_token_vec(output, rng);
            }
            if flag && e.2 {
                return;
            }
        }
    }
}

use std::collections::{HashMap, HashSet};
impl RawBms {
    pub fn parse(source: &str) -> RawBms {
        use token::*;
        use winnow::prelude::*;
        let token_stream = lex::lex(source);
        let all_wav_files = token_stream
            .iter()
            .filter_map(|t| {
                if let Token::Command(Command::Wav(_, file)) = t {
                    Some(file.clone())
                }
                else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        RawBms {
            raw_bms: parse::block
                .parse_next(&mut token_stream.as_slice())
                .unwrap(),
            all_wav_files,
        }
    }
    pub fn all_wav_files(&self) -> &HashSet<String> {
        &self.all_wav_files
    }
    pub fn make_bms(&self, mut rng: impl rand::RngCore) -> Bms {
        use token::Command::*;
        let mut commands = vec![];
        self.raw_bms.get_token_vec(&mut commands, &mut rng);

        let base62 = commands.iter().any(|c| matches!(c, Base62));

        let mut bms = Bms {
            main_data: Vec::with_capacity(1000),
            ..Default::default()
        };

        let convert_channel_vec = |ch_vec: &[Channel]| {
            ch_vec
                .iter()
                .map(|ch| ch.to_base_36_or_62(base62))
                .collect::<Vec<_>>()
        };
        for c in commands {
            match c {
                MainData(measure, data) => {
                    if bms.main_data.len() <= *measure {
                        bms.main_data
                            .resize_with(measure + 1, Default::default);
                    }
                    let measure = &mut bms.main_data[*measure];
                    use token::MainDataValue::*;
                    #[rustfmt::skip]
                    match data {
                        Bgm(data) =>
                            measure.bgm.push(convert_channel_vec(data)),
                        Length(n) =>
                            measure.length = *n,
                        Bga(data) =>
                            measure.bga.push(convert_channel_vec(data)),
                        Bpm(data) =>
                            measure.bpm.push(data),
                        BgaPoor(data) =>
                            measure.bga_poor.push(convert_channel_vec(data)),
                        BgaLayer(data) =>
                            measure.bga_layer.push(convert_channel_vec(data)),
                        ExBpm(data) =>
                            measure.ex_bpm.push(convert_channel_vec(data)),
                        Stop(data) =>
                            measure.stop.push(convert_channel_vec(data)),
                        BgaLayer2(data) =>
                            measure.bga_layer2.push(convert_channel_vec(data)),
                        ExRank(data) =>
                            measure.ex_rank.push(convert_channel_vec(data)),
                        BgaAlpha(data) =>
                            measure.bga_alpha.push(data),
                        BgaLayerAlpha(data) =>
                            measure.bga_layer_alpha.push(data),
                        BgaLayer2Alpha(data) =>
                            measure.bga_layer2_alpha.push(data),
                        BgaPoorAlpha(data) =>
                            measure.bga_poor_alpha.push(data),
                        Note(ch, data) =>
                            measure.notes.entry(*ch).or_default().push(convert_channel_vec(data)),
                        InvisibleNote(ch, data) =>
                            measure.invisible_notes.entry(*ch).or_default().push(convert_channel_vec(data)),
                        LongNote(ch, data) =>
                            measure.long_notes.entry(*ch).or_default().push(convert_channel_vec(data)),
                        Text(data) =>
                            measure.text.push(convert_channel_vec(data)),
                        BgaArgb(data) =>
                            measure.bga_argb.push(convert_channel_vec(data)),
                        BgaLayerArgb(data) =>
                            measure.bga_layer_argb.push(convert_channel_vec(data)),
                        BgaLayer2Argb(data) =>
                            measure.bga_layer2_argb.push(convert_channel_vec(data)),
                        BgaPoorArgb(data) =>
                            measure.bga_poor_argb.push(convert_channel_vec(data)),
                        SwitchBga(data) =>
                            measure.switch_bga.push(convert_channel_vec(data)),
                        Option(data) =>
                            measure.option.push(convert_channel_vec(data)),
                        Landmine(ch, data) =>
                            measure.landmine.entry(*ch).or_default().push(data),
                        Scroll(data) =>
                            measure.scroll.push(convert_channel_vec(data)),
                        Speed(data) =>
                            measure.speed.push(convert_channel_vec(data)),
                        Other(ch, data) =>
                            measure.other.push((*ch, data)),
                    };
                }
                Player(n) => {
                    bms.deprecated.player = Some(match n {
                        1 => PlayType::SinglePlay,
                        2 => PlayType::CouplePlay,
                        3 => PlayType::DoublePlay,
                        4 => PlayType::BattlePlay,
                        _ => unreachable!(),
                    });
                }
                Rank(n) => bms.rank = Some(*n),
                DefExRank(n) => bms.def_ex_rank = Some(*n),
                ExRank(ch, n) => {
                    bms.uncommon
                        .ex_rank
                        .insert(ch.to_base_36_or_62(base62), *n);
                }
                Total(n) => bms.total = Some(*n),
                VolumeWav(n) => bms.volume_wav = Some(*n),
                StageFile(s) => bms.stage_file = Some(s),
                Banner(s) => bms.banner = Some(s),
                BackBmp(s) => bms.back_bmp = Some(s),
                CharacterFile(s) => bms.uncommon.character_file = Some(s),
                PlayLevel(n) => bms.play_level = Some(*n),
                Difficulty(n) => bms.difficulty = Some(*n),
                Title(s) => bms.title = Some(s),
                SubTitle(s) => bms.sub_title.push(s),
                Artist(s) => bms.artist = Some(s),
                SubArtist(s) => bms.sub_artist.push(s),
                Maker(s) => bms.uncommon.maker = Some(s),
                Genre(s) => bms.genre = Some(s),
                Comment(s) => bms.uncommon.comment.push(s),
                Text(ch, s) => {
                    bms.uncommon.text.insert(ch.to_base_36_or_62(base62), s);
                }
                PathWav(s) => bms.uncommon.path_wav = Some(s),
                Bpm(n) => bms.bpm = Some(*n),
                ExBpm(ch, n) => {
                    bms.ex_bpm.insert(ch.to_base_36_or_62(base62), *n);
                }
                BaseBpm(n) => bms.deprecated.base_bpm = Some(*n),
                Stop(ch, n) => {
                    bms.stop.insert(ch.to_base_36_or_62(base62), *n);
                }
                Stp(x, y, z) => bms.uncommon.stp.push((*x, *y, *z)),
                LnMode(n) => bms.ln_mode = Some(*n),
                LnType(n) => bms.ln_type = Some(*n),
                LnObject(ch) => {
                    bms.ln_object.insert(ch.to_base_36_or_62(base62));
                }
                OctFp => bms.uncommon.oct_fp = true,
                Option(name, opt) => bms.uncommon.option.push((name, opt)),
                ChangeOption(ch, name, opt) => {
                    bms.uncommon
                        .change_option
                        .insert(ch.to_base_36_or_62(base62), (name, opt));
                }
                Wav(ch, s) => {
                    bms.wav.insert(ch.to_base_36_or_62(base62), s);
                }
                WavCommand(opt, ch, v) => {
                    bms.uncommon.wav_command.push((
                        *opt,
                        ch.to_base_36_or_62(base62),
                        *v,
                    ));
                }
                ExWav(ch, opt, s) => {
                    bms.uncommon
                        .ex_wav
                        .insert(ch.to_base_36_or_62(base62), (opt, s));
                }
                Cdda(n) => bms.uncommon.cdda = Some(*n),
                MidiFile(s) => bms.uncommon.midi_file = Some(s),
                Bmp(ch, s) => {
                    bms.bmp.insert(ch.to_base_36_or_62(base62), s);
                }
                ExBmp(ch, argb, s) => {
                    bms.uncommon
                        .ex_bmp
                        .insert(ch.to_base_36_or_62(base62), (argb, s));
                }
                Bga(ch, bmp, pos) => {
                    bms.uncommon.bga.insert(
                        ch.to_base_36_or_62(base62),
                        (bmp.to_base_36_or_62(base62), pos),
                    );
                }
                AtBga(ch, bmp, pos) => {
                    bms.uncommon.at_bga.insert(
                        ch.to_base_36_or_62(base62),
                        (bmp.to_base_36_or_62(base62), pos),
                    );
                }
                PoorBga(n) => bms.uncommon.poor_bga = Some(*n),
                SwitchBga(ch, fr, time, line, r#loop, argb, data) => {
                    bms.deprecated.switch_bga.insert(
                        ch.to_base_36_or_62(base62),
                        (*fr, *time, line.to_base_36(), *r#loop, argb, data),
                    );
                }
                Argb(ch, argb) => {
                    bms.uncommon.argb.insert(ch.to_base_36_or_62(base62), argb);
                }
                VideoFile(s) => bms.uncommon.video_file = Some(s),
                VideoFps(n) => bms.uncommon.video_fps = Some(*n),
                VideoColors(n) => bms.uncommon.video_colors = Some(*n),
                VideoDelay(n) => bms.uncommon.video_delay = Some(*n),
                Movie(s) => bms.uncommon.movie = Some(s),
                Seek(ch, n) => {
                    bms.deprecated.seek.insert(ch.to_base_36_or_62(base62), *n);
                }
                ExCharacter(sprite_num, bmp, trim_rect, offset, abs_pos) => {
                    bms.uncommon.ex_character = Some(super::bms::ExCharacter {
                        sprite_num: *sprite_num,
                        bmp: *bmp,
                        trim_rect,
                        offset: offset.as_ref(),
                        abs_pos: abs_pos.as_ref(),
                    });
                }
                Url(s) => bms.url = Some(s),
                Email(s) => bms.email = Some(s),
                Scroll(ch, n) => {
                    bms.scroll.insert(ch.to_base_36_or_62(base62), *n);
                }
                Speed(ch, n) => {
                    bms.speed.insert(ch.to_base_36_or_62(base62), *n);
                }
                Preview(s) => bms.preview = Some(s),
                Other(command, value) => {
                    bms.uncommon.other.push((command, value));
                }
                _ => (),
            }
        }
        bms
    }
}

/// ランダムを考慮したBMS
#[derive(Default, Debug, PartialEq)]
pub struct Bms<'a> {
    /// メインデータ
    ///
    /// mmmcc:chchch...
    ///
    /// mmm : 小節数 [0-9]
    ///
    /// cc : チャンネル [0-9, A-Z, a-z]
    ///
    /// chchch.. : メインのデータで、2文字一組として扱う [0-9, A-Z, a-z]
    pub main_data: Vec<MainData<'a>>,
    /// 判定幅
    ///
    /// 2を基本とするのが主流だが、その幅は実装依存
    pub rank: Option<i32>,
    /// より細かい判定幅
    ///
    /// 100をRank2と同じとするのが主流だが、
    /// Rankの1に相当する数が実装依存
    pub def_ex_rank: Option<f64>,
    /// ゲージ増加の総数
    ///
    /// 全て最良判定のときのゲージの増加量
    pub total: Option<f64>,
    /// 譜面全体の音量
    pub volume_wav: Option<f64>,
    /// ロード画面に表示する画像
    pub stage_file: Option<&'a str>,
    /// 選曲画面やリザルト画面に表示する横長の画像
    pub banner: Option<&'a str>,
    /// ステージファイルに重ねる画像
    ///
    /// 選曲後カバー等の調整をする画面で、
    /// ステージファイルにタイトルの代わりに重ねる
    pub back_bmp: Option<&'a str>,
    /// レベル
    pub play_level: Option<i32>,
    /// 難易度
    ///
    /// 主流な名付けは
    ///
    /// 1 : EASY, BEGINNER, LIGHT ...
    ///
    /// 2 : NORMAL, STANDARD ...
    ///
    /// 3 : HARD, HYPER ...
    ///
    /// 4 : EX, ANOTHER ...
    ///
    /// 5 : BLACK_ANOTHER, INSANE, 発狂 ...
    pub difficulty: Option<i32>,
    /// タイトル
    pub title: Option<&'a str>,
    /// サブタイトル
    pub sub_title: Vec<&'a str>,
    /// アーティスト
    pub artist: Option<&'a str>,
    /// サブアーティスト
    pub sub_artist: Vec<&'a str>,
    /// ジャンル名
    pub genre: Option<&'a str>,
    /// 基本BPM
    ///
    /// 曲選択時の表示や曲の最初のBPMとして使う
    pub bpm: Option<f64>,
    /// BPM変化用
    ///
    /// BPM変化時に256以上の値を指定したいとき、
    /// 08チャンネルでidを指定する
    pub ex_bpm: HashMap<usize, f64>,
    /// 停止
    ///
    /// 譜面を停止するときに09チャンネルでidを指定する
    pub stop: HashMap<usize, f64>,
    /// LNの判定方法
    ///
    /// 1 : LN, 始点判定が基準で、最後まで押し切らないとPOOR
    ///
    /// 2 : CN, 始点判定と終点判定がある
    ///
    /// 3 : HCN, CHの判定に、押している間16分ごとに判定を追加したもの
    pub ln_mode: Option<i32>,
    /// LNの指定方法
    ///
    /// 1 : 主流の指定方法
    ///
    /// メインデータの0以外のidをLNの始点終点の繰り返しとして解析
    ///
    /// 2 : 非推奨な指定方法
    ///
    /// メインデータの0以外のidが続く部分をLNとして解析
    ///
    /// 例
    ///
    /// 1 : 0011000000002200
    ///
    /// 2 : 0011111111220000
    pub ln_type: Option<i32>,
    /// LNの音を鳴らす終点idの指定
    ///
    /// LNTYPE 1 で終点として使うと、音が鳴るようになる
    pub ln_object: HashSet<usize>,
    /// 音声ファイル
    ///
    /// 詳細はメインデータ等に記載
    pub wav: HashMap<usize, &'a str>,
    /// 画像ファイル
    ///
    /// 詳細はメインデータ等に記載
    pub bmp: HashMap<usize, &'a str>,
    /// BMS制作者のホームページ
    pub url: Option<&'a str>,
    /// BMS制作者のEメール
    pub email: Option<&'a str>,
    /// 譜面速度
    pub scroll: HashMap<usize, f64>,
    /// 譜面速度
    ///
    /// SCROLLと違って線形補間がされる
    pub speed: HashMap<usize, f64>,
    /// 曲選択時に流れる音声
    ///
    /// ここで指定されていなければ、
    /// previewと名前が付いた音声ファイルを流すのが主流
    pub preview: Option<&'a str>,
    /// あまり使われないコマンド
    pub uncommon: UncommonHeader<'a>,
    /// 非推奨のコマンド
    pub deprecated: DeprecatedHeader<'a>,
}

/// 一小節ごとのメインデータ
///
/// 同じ小節、同じチャンネルが複数行定義された場合に対応
///
/// bgmとoptionのみ複数行に対応している場合が多い
#[derive(Debug, PartialEq)]
pub struct MainData<'a> {
    /// BGM
    pub bgm: Vec<Vec<usize>>,
    /// 一小節の長さ
    ///
    /// 1が基準
    pub length: f64,
    /// BPM
    ///
    /// 1から255まで
    pub bpm: Vec<&'a [Option<f64>]>,
    /// BGA
    pub bga: Vec<Vec<usize>>,
    /// POOR BGA
    pub bga_poor: Vec<Vec<usize>>,
    /// BGA LAYER
    pub bga_layer: Vec<Vec<usize>>,
    /// EXBPM
    pub ex_bpm: Vec<Vec<usize>>,
    /// 停止
    pub stop: Vec<Vec<usize>>,
    /// BGA LAYER2
    pub bga_layer2: Vec<Vec<usize>>,
    /// BGA不透明度
    pub bga_alpha: Vec<&'a [u8]>,
    /// BGA LAYER不透明度
    pub bga_layer_alpha: Vec<&'a [u8]>,
    /// BGA LAYER2不透明度
    pub bga_layer2_alpha: Vec<&'a [u8]>,
    /// POOR BGA不透明度
    pub bga_poor_alpha: Vec<&'a [u8]>,
    /// ノーツ
    ///
    /// チャンネルを36進数で解釈した値をキーにした`HashMap`
    pub notes: HashMap<usize, Vec<Vec<usize>>>,
    /// 不可視ノーツ
    ///
    /// チャンネルを36進数で解釈した値をキーにした`HashMap`
    pub invisible_notes: HashMap<usize, Vec<Vec<usize>>>,
    /// ロングノーツ
    ///
    /// チャンネルを36進数で解釈した値をキーにした`HashMap`
    pub long_notes: HashMap<usize, Vec<Vec<usize>>>,
    /// テキスト
    pub text: Vec<Vec<usize>>,
    /// EXRANK
    pub ex_rank: Vec<Vec<usize>>,
    /// BGA aRGB
    pub bga_argb: Vec<Vec<usize>>,
    /// BGA LAYER aRGB
    pub bga_layer_argb: Vec<Vec<usize>>,
    /// BGA LAYER2 aRGB
    pub bga_layer2_argb: Vec<Vec<usize>>,
    /// POOR BGA aRGB
    pub bga_poor_argb: Vec<Vec<usize>>,
    /// SWBGA
    pub switch_bga: Vec<Vec<usize>>,
    /// オプション
    pub option: Vec<Vec<usize>>,
    /// 地雷
    pub landmine: HashMap<usize, Vec<&'a [f64]>>,
    /// スクロール速度
    pub scroll: Vec<Vec<usize>>,
    /// ノーツ速度
    pub speed: Vec<Vec<usize>>,

    /// その他
    pub other: Vec<(usize, &'a str)>,
}
impl Default for MainData<'_> {
    fn default() -> Self {
        MainData {
            bgm: vec![],
            length: 1.0,
            bpm: vec![],
            bga: vec![],
            bga_poor: vec![],
            bga_layer: vec![],
            ex_bpm: vec![],
            stop: vec![],
            bga_layer2: vec![],
            bga_alpha: vec![],
            bga_layer_alpha: vec![],
            bga_layer2_alpha: vec![],
            bga_poor_alpha: vec![],
            notes: HashMap::new(),
            invisible_notes: HashMap::new(),
            long_notes: HashMap::new(),
            text: vec![],
            ex_rank: vec![],
            bga_argb: vec![],
            bga_layer_argb: vec![],
            bga_layer2_argb: vec![],
            bga_poor_argb: vec![],
            switch_bga: vec![],
            option: vec![],
            landmine: HashMap::new(),
            scroll: vec![],
            speed: vec![],
            other: vec![],
        }
    }
}

/// あまり使われないヘッダー
#[derive(Default, Debug, PartialEq)]
pub struct UncommonHeader<'a> {
    /// メインデータで指定可能な判定幅の設定
    ///
    /// 値はDefExRankと同じ扱いをすることが主流だが、DefExRankと同じように判定幅が実装依存
    pub ex_rank: HashMap<usize, f64>,
    pub character_file: Option<&'a str>,
    /// 譜面制作者
    pub maker: Option<&'a str>,
    /// 曲選択時に表示するコメント
    pub comment: Vec<&'a str>,
    /// プレイ中に表示するテキスト
    pub text: HashMap<usize, &'a str>,
    /// 音声ファイルを読み込むときに参照するフォルダ
    pub path_wav: Option<&'a str>,
    pub stp: Vec<(usize, u32, f64)>,
    pub oct_fp: bool,
    pub option: Vec<(&'a str, &'a str)>,
    pub change_option: HashMap<usize, (&'a str, &'a str)>,
    pub wav_command: Vec<(i32, usize, f64)>,
    pub ex_wav: HashMap<usize, (&'a [Option<f64>; 3], &'a str)>,
    pub cdda: Option<u32>,
    pub midi_file: Option<&'a str>,
    pub ex_bmp: HashMap<usize, (&'a [u8; 4], &'a str)>,
    pub bga: HashMap<usize, (usize, &'a [[f64; 2]; 3])>,
    pub at_bga: HashMap<usize, (usize, &'a [[f64; 2]; 3])>,
    pub poor_bga: Option<i32>,
    pub argb: HashMap<usize, &'a [u8; 4]>,
    pub video_file: Option<&'a str>,
    pub video_fps: Option<f64>,
    pub video_colors: Option<u32>,
    pub video_delay: Option<u32>,
    pub movie: Option<&'a str>,
    pub ex_character: Option<ExCharacter<'a>>,
    /// その他のコマンド
    pub other: Vec<(&'a str, &'a str)>,
}

/// Extended-Characterファイル
#[derive(Debug, PartialEq)]
pub struct ExCharacter<'a> {
    pub sprite_num: u32,
    pub bmp: usize,
    pub trim_rect: &'a [[f64; 2]; 2],
    pub offset: Option<&'a [f64; 2]>,
    pub abs_pos: Option<&'a [f64; 2]>,
}

/// 非推奨のヘッダー
#[derive(Default, Debug, PartialEq)]
pub struct DeprecatedHeader<'a> {
    /// プレイ方法
    ///
    /// 主にメインデータから解析するのが主流
    pub player: Option<PlayType>,
    /// スクロール速度の基準BPM
    ///
    /// オプション側で基準とするBPMを最大、最小、最長、最初のBPMに合わせられるようにするべき
    pub base_bpm: Option<f64>,
    /// キーを押したときに表示するBGA
    ///
    /// 試験的に追加された
    pub switch_bga:
        HashMap<usize, (f64, f64, usize, bool, &'a [u8; 4], &'a [Channel])>,
    /// ビデオの再生位置を調整
    ///
    /// 提案されたプレイヤーで削除済み
    pub seek: HashMap<usize, f64>,
}

/// プレイ方式
#[derive(Debug, PartialEq)]
pub enum PlayType {
    SinglePlay,
    CouplePlay,
    DoublePlay,
    BattlePlay,
}
