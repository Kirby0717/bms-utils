use ordered_float::NotNan;
pub type FinF64 = NotNan<f64>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Token {
    Command(Command),
    ControlFlow(ControlFlow),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Command {
    MainData(usize, MainDataValue),
    Player(i32),
    Rank(i32),
    DefExRank(FinF64),
    ExRank(Channel, FinF64),
    Total(FinF64),
    VolumeWav(FinF64),
    StageFile(String),
    Banner(String),
    BackBmp(String),
    CharacterFile(String),
    PlayLevel(i32),
    Difficulty(i32),
    Title(String),
    SubTitle(String),
    Artist(String),
    SubArtist(String),
    Maker(String),
    Genre(String),
    Comment(String),
    Text(Channel, String),
    PathWav(String),
    Bpm(FinF64),
    ExBpm(Channel, FinF64),
    BaseBpm(FinF64),
    Stop(Channel, FinF64),
    Stp(usize, u32, FinF64),
    LnMode(i32),
    LnType(i32),
    LnObject(Channel),
    OctFp,
    Option(String, String),
    ChangeOption(Channel, String, String),
    Wav(Channel, String),
    WavCommand(i32, Channel, FinF64),
    ExWav(Channel, [Option<FinF64>; 3], String),
    Cdda(u32),
    MidiFile(String),
    Bmp(Channel, String),
    ExBmp(Channel, [u8; 4], String),
    Bga(Channel, Channel, [[FinF64; 2]; 3]),
    AtBga(Channel, Channel, [[FinF64; 2]; 3]),
    PoorBga(i32),
    SwitchBga(
        Channel,
        FinF64,
        FinF64,
        Channel,
        bool,
        [u8; 4],
        Vec<Channel>,
    ),
    Argb(Channel, [u8; 4]),
    VideoFile(String),
    VideoFps(FinF64),
    VideoColors(u32),
    VideoDelay(u32),
    Movie(String),
    Seek(Channel, FinF64),
    ExCharacter(
        u32,
        usize,
        [[FinF64; 2]; 2],
        Option<[FinF64; 2]>,
        Option<[FinF64; 2]>,
    ),
    Url(String),
    Email(String),
    Scroll(Channel, FinF64),
    Speed(Channel, FinF64),
    Preview(String),
    /// 旧型BPM変更( 16進数 )と地雷( 36進数 )以外の
    /// WAV BMP BPM STOP SCROLLのチャンネルの指定と参照を62進数で解釈する
    Base62,
    /// その他のコマンド
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MainDataValue {
    Bgm(Vec<Channel>),
    Length(FinF64),
    Bga(Vec<Channel>),
    Bpm(Vec<Option<FinF64>>),
    BgaPoor(Vec<Channel>),
    BgaLayer(Vec<Channel>),
    ExBpm(Vec<Channel>),
    Stop(Vec<Channel>),
    BgaLayer2(Vec<Channel>),
    ExRank(Vec<Channel>),
    BgaAlpha(Vec<u8>),
    BgaLayerAlpha(Vec<u8>),
    BgaLayer2Alpha(Vec<u8>),
    BgaPoorAlpha(Vec<u8>),
    Note(usize, Vec<Channel>),
    InvisibleNote(usize, Vec<Channel>),
    LongNote(usize, Vec<Channel>),
    Text(Vec<Channel>),
    BgaArgb(Vec<Channel>),
    BgaLayerArgb(Vec<Channel>),
    BgaLayer2Argb(Vec<Channel>),
    BgaPoorArgb(Vec<Channel>),
    SwitchBga(Vec<Channel>),
    Option(Vec<Channel>),
    Landmine(usize, Vec<FinF64>),
    Scroll(Vec<Channel>),
    Speed(Vec<Channel>),
    Other(usize, String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ControlFlow {
    Random(u128),
    SetRandom(u128),
    EndRandom,
    If(u128),
    ElseIf(u128),
    Else,
    EndIf,
    Switch(u128),
    SetSwitch(u128),
    EndSwitch,
    Case(u128),
    Skip,
    Default,
}

/// 整数・アルファベット大文字小文字の二文字分
///
/// 36進数か62進数かの確定前の型
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Channel([i32; 2]);
impl std::str::FromStr for Channel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or(())
    }
}
impl Channel {
    const fn parse_base_62(c: u8) -> i32 {
        if c.is_ascii_digit() {
            (c - b'0') as i32
        }
        else if c.is_ascii_uppercase() {
            (c - b'A') as i32 + 10
        }
        else if c.is_ascii_lowercase() {
            (c - b'a') as i32 + 36
        }
        else {
            0
        }
    }
    pub fn new(s: &str) -> Option<Channel> {
        if s.chars().take(2).all(|c| c.is_ascii_alphanumeric()) {
            Some(Channel::new_unchecked(s))
        }
        else {
            None
        }
    }
    pub const fn new_unchecked(s: &str) -> Channel {
        let s = s.as_bytes();
        match s.len() {
            0 => Channel([0, 0]),
            1 => Channel([0, Self::parse_base_62(s[0])]),
            2.. => {
                Channel([Self::parse_base_62(s[0]), Self::parse_base_62(s[1])])
            }
        }
    }
    pub const fn to_base_36_or_62(&self, flag: bool) -> usize {
        if flag {
            self.to_base_62()
        }
        else {
            self.to_base_36()
        }
    }
    const fn convert_base_36(n: i32) -> i32 {
        if n < 36 { n } else { n - 26 }
    }
    pub const fn to_base_36(&self) -> usize {
        (36 * Self::convert_base_36(self.0[0])
            + Self::convert_base_36(self.0[1])) as usize
    }
    pub const fn to_base_62(&self) -> usize {
        (62 * self.0[0] + self.0[1]) as usize
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn channel_to_number() {
        use super::Channel;

        let zero1 = Channel::new("0").unwrap();
        let zero2 = Channel::new("00").unwrap();
        let one = Channel::new("1").unwrap();
        let ua = Channel::new("A").unwrap();
        let uz = Channel::new("Z").unwrap();
        let la = Channel::new("a").unwrap();
        let lz = Channel::new("z").unwrap();
        let ten = Channel::new("10").unwrap();
        assert!(Channel::new("テスト").is_none());

        assert_eq!(zero1.to_base_36(), 0);
        assert_eq!(zero1.to_base_62(), 0);

        assert_eq!(zero2.to_base_36(), 0);
        assert_eq!(zero2.to_base_62(), 0);

        assert_eq!(one.to_base_36(), 1);
        assert_eq!(one.to_base_62(), 1);

        assert_eq!(ua.to_base_36(), 10);
        assert_eq!(ua.to_base_62(), 10);

        assert_eq!(uz.to_base_36(), 35);
        assert_eq!(uz.to_base_62(), 35);

        assert_eq!(la.to_base_36(), 10);
        assert_eq!(la.to_base_62(), 36);

        assert_eq!(lz.to_base_36(), 35);
        assert_eq!(lz.to_base_62(), 61);

        assert_eq!(ten.to_base_36(), 36);
        assert_eq!(ten.to_base_62(), 62);
    }
}
