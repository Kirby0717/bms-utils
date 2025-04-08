# bms-utils

BMSのライブラリです。

BMSファイルの読み書きが出来ます

## 拡張子がbms,bme,bml,pmsのファイル
```rust
// 読み込み

let bms_str = r"
#PLAYER 1
#GENRE ジャンル
#TITLE タイトル
#ARTIST 制作者
#BPM 180
#PLAYLEVEL 12
#RANK 3

#SUBTITLE サブタイトル
#SUBARTIST サブ制作者
#STAGEFILE ステージ画像
#BANNER バナー画像
#BACKBMP タイトル文字画像

#DIFFICULTY 4
#TOTAL 400
#LNOBJ ZZ
#PREVIEW preview.wav
#LNMODE 2
";
// ランダム要素を確定していない状態のBMSを作成
let rawbms = RawBms::parse(bms_str);
// ランダム要素を確定させる
// この時、疑似乱数生成器を渡す
let bms = rawbms.make_bms(rng);

// 書き込み

// !!!開発中!!!
```
## Bmsonファイル
```rust
// 読み込み

let bmson = Bmson::parse(&bmson_string).unwrap();

// 書き込み

// 改行が無く、小さい長さの文字列へ
let bmson_string = bmson.to_string().unwrap();

// 改行やインデントがなされ、読みやすい文字列へ
let bmson_string = bmson.to_string_pretty().unwrap();
```

License: MIT OR Apache-2.0
