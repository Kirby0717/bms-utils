#![allow(dead_code)]

use super::token::{ControlFlow::*, Token, Token::ControlFlow};
use super::*;
use winnow::combinator::{opt, preceded};
use winnow::token::one_of;
use winnow::{
    combinator::{alt, repeat},
    prelude::*,
};

pub(crate) fn block(input: &mut &[Token]) -> ModalResult<BmsBlock> {
    Ok(BmsBlock(
        repeat(
            0..,
            alt((
                one_of(|t| matches!(t, Token::Command(_)))
                    .map(|t| match t {
                        Token::Command(c) => c,
                        _ => unreachable!(),
                    })
                    .map(BmsElement::Command),
                random_block.map(BmsElement::Random),
                switch_block.map(BmsElement::Switch),
            )),
        )
        .parse_next(input)?,
    ))
}
fn random_block(input: &mut &[Token]) -> ModalResult<BmsRandomBlock> {
    let (n, e, _) = (
        one_of(|t| matches!(t, ControlFlow(Random(_) | SetRandom(_)))).map(
            |t| match t {
                ControlFlow(Random(n)) => RandomValue::Max(n),
                ControlFlow(SetRandom(n)) => RandomValue::Set(n),
                _ => unreachable!(),
            },
        ),
        repeat(
            0..,
            alt((
                if_block.map(BmsRandomElement::IfBlock),
                block
                    .verify(|b| !b.0.is_empty())
                    .map(BmsRandomElement::Block),
            )),
        ),
        opt(one_of(ControlFlow(EndRandom))),
    )
        .parse_next(input)?;
    Ok(BmsRandomBlock(n, e))
}
fn if_block(input: &mut &[Token]) -> ModalResult<BmsIfBlock> {
    let mut if_block = BmsIfBlock::default();
    if_block.r#if.push(
        (
            one_of(|t| matches!(t, ControlFlow(If(_)))).map(|t| match t {
                ControlFlow(If(n)) => n,
                _ => unreachable!(),
            }),
            block,
        )
            .parse_next(input)?,
    );
    if_block.r#if.extend::<Vec<_>>(
        repeat(
            0..,
            (
                one_of(|t| matches!(t, ControlFlow(ElseIf(_)))).map(
                    |t| match t {
                        ControlFlow(ElseIf(n)) => n,
                        _ => unreachable!(),
                    },
                ),
                block,
            ),
        )
        .parse_next(input)?,
    );
    if_block.r#else =
        opt(preceded(one_of(ControlFlow(Else)), block)).parse_next(input)?;
    one_of(ControlFlow(EndIf)).parse_next(input)?;
    Ok(if_block)
}
fn switch_block(input: &mut &[Token]) -> ModalResult<BmsSwitchBlock> {
    let (n, b, _): (_, Vec<_>, _) = (
        one_of(|t| matches!(t, ControlFlow(Switch(_) | SetSwitch(_)))).map(
            |t| match t {
                ControlFlow(Switch(n)) => RandomValue::Max(n),
                ControlFlow(SetSwitch(n)) => RandomValue::Set(n),
                _ => unreachable!(),
            },
        ),
        repeat(0.., case_block),
        one_of(ControlFlow(EndSwitch)),
    )
        .parse_next(input)?;
    let default_set = b
        .iter()
        .filter_map(|BmsCaseBlock(l, _, _)| {
            if let SwitchLabel::Case(n) = l {
                Some(*n)
            }
            else {
                None
            }
        })
        .collect();
    Ok(BmsSwitchBlock(n, b, default_set))
}
fn case_block(input: &mut &[Token]) -> ModalResult<BmsCaseBlock> {
    let (l, b, s) = (
        one_of(|t| matches!(t, ControlFlow(Case(_) | Default))).map(
            |t| match t {
                ControlFlow(Case(n)) => SwitchLabel::Case(n),
                ControlFlow(Default) => SwitchLabel::Default,
                _ => unreachable!(),
            },
        ),
        block,
        opt(one_of(ControlFlow(Skip))).map(|t| t.is_some()),
    )
        .parse_next(input)?;
    Ok(BmsCaseBlock(l, b, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        use token::{
            Command::{Genre, Player, Title},
            Token::Command,
        };
        let empty_token_stream = vec![];
        assert_eq!(
            block
                .parse_next(&mut empty_token_stream.as_slice())
                .unwrap(),
            BmsBlock(vec![])
        );
        let token_stream = vec![
            Command(Player(1)),
            Command(Genre("ジャンル".to_string())),
            Command(Title("タイトル".to_string())),
        ];
        assert_eq!(
            block.parse_next(&mut token_stream.as_slice()).unwrap(),
            BmsBlock(vec![
                BmsElement::Command(Player(1)),
                BmsElement::Command(Genre("ジャンル".to_string())),
                BmsElement::Command(Title("タイトル".to_string())),
            ])
        );
    }

    #[test]
    fn random() {
        use token::{
            Command::{Artist, Genre, PlayLevel, Rank, Title},
            ControlFlow::{
                Else, ElseIf, EndIf, EndRandom, If, Random, SetRandom,
            },
            Token::{Command, ControlFlow},
        };
        let token_stream = vec![
            Command(PlayLevel(12)),
            ControlFlow(Random(10)),
            Command(Genre("ジャンル".to_string())),
            ControlFlow(If(1)),
            Command(Title("タイトル1".to_string())),
            ControlFlow(ElseIf(2)),
            Command(Title("タイトル2".to_string())),
            ControlFlow(ElseIf(4)),
            Command(Title("タイトル4".to_string())),
            ControlFlow(Else),
            Command(Title("タイトル*".to_string())),
            ControlFlow(Random(100)),
            ControlFlow(EndRandom),
            ControlFlow(EndIf),
            Command(Artist("アーティスト".to_string())),
            ControlFlow(If(1)),
            ControlFlow(EndIf),
            ControlFlow(If(2)),
            ControlFlow(ElseIf(3)),
            ControlFlow(EndIf),
            ControlFlow(If(4)),
            ControlFlow(Else),
            ControlFlow(EndIf),
            ControlFlow(EndRandom),
            Command(Rank(3)),
            ControlFlow(SetRandom(123456789012345678901234567890)),
        ];
        assert_eq!(
            block.parse_next(&mut token_stream.as_slice()).unwrap(),
            BmsBlock(vec![
                BmsElement::Command(PlayLevel(12)),
                BmsElement::Random(BmsRandomBlock(
                    RandomValue::Max(10),
                    vec![
                        BmsRandomElement::Block(BmsBlock(vec![
                            BmsElement::Command(Genre("ジャンル".to_string()))
                        ])),
                        BmsRandomElement::IfBlock(BmsIfBlock {
                            r#if: vec![
                                (
                                    1,
                                    BmsBlock(vec![BmsElement::Command(Title(
                                        "タイトル1".to_string()
                                    ))])
                                ),
                                (
                                    2,
                                    BmsBlock(vec![BmsElement::Command(Title(
                                        "タイトル2".to_string()
                                    ))])
                                ),
                                (
                                    4,
                                    BmsBlock(vec![BmsElement::Command(Title(
                                        "タイトル4".to_string()
                                    )),])
                                ),
                            ],
                            r#else: Some(BmsBlock(vec![
                                BmsElement::Command(Title(
                                    "タイトル*".to_string()
                                )),
                                BmsElement::Random(BmsRandomBlock(
                                    RandomValue::Max(100),
                                    vec![]
                                ))
                            ])),
                        }),
                        BmsRandomElement::Block(BmsBlock(vec![
                            BmsElement::Command(Artist(
                                "アーティスト".to_string()
                            ))
                        ])),
                        BmsRandomElement::IfBlock(BmsIfBlock {
                            r#if: vec![(1, BmsBlock(vec![])),],
                            r#else: None,
                        }),
                        BmsRandomElement::IfBlock(BmsIfBlock {
                            r#if: vec![
                                (2, BmsBlock(vec![])),
                                (3, BmsBlock(vec![])),
                            ],
                            r#else: None,
                        }),
                        BmsRandomElement::IfBlock(BmsIfBlock {
                            r#if: vec![(4, BmsBlock(vec![])),],
                            r#else: Some(BmsBlock(vec![])),
                        }),
                    ]
                )),
                BmsElement::Command(Rank(3)),
                BmsElement::Random(BmsRandomBlock(
                    RandomValue::Set(123456789012345678901234567890),
                    vec![],
                )),
            ])
        );
    }

    #[test]
    fn switch() {
        use token::{
            Command::{PlayLevel, Title},
            ControlFlow::{Case, EndSwitch, SetSwitch, Skip, Switch},
            Token::{Command, ControlFlow},
        };
        let token_stream = vec![
            Command(PlayLevel(12)),
            ControlFlow(Switch(10)),
            ControlFlow(Case(1)),
            Command(Title("タイトル1".to_string())),
            ControlFlow(Case(2)),
            Command(Title("タイトル2".to_string())),
            ControlFlow(Skip),
            ControlFlow(Case(4)),
            Command(Title("タイトル4".to_string())),
            ControlFlow(Skip),
            ControlFlow(Default),
            Command(Title("タイトル*".to_string())),
            ControlFlow(Switch(100)),
            ControlFlow(EndSwitch),
            ControlFlow(EndSwitch),
            ControlFlow(SetSwitch(123456789012345678901234567890)),
            ControlFlow(Case(10)),
            ControlFlow(Skip),
            ControlFlow(Default),
            ControlFlow(Case(20)),
            ControlFlow(EndSwitch),
        ];
        assert_eq!(
            block.parse_next(&mut token_stream.as_slice()).unwrap(),
            BmsBlock(vec![
                BmsElement::Command(PlayLevel(12)),
                BmsElement::Switch(BmsSwitchBlock(
                    RandomValue::Max(10),
                    vec![
                        BmsCaseBlock(
                            SwitchLabel::Case(1),
                            BmsBlock(vec![BmsElement::Command(Title(
                                "タイトル1".to_string()
                            ))]),
                            false
                        ),
                        BmsCaseBlock(
                            SwitchLabel::Case(2),
                            BmsBlock(vec![BmsElement::Command(Title(
                                "タイトル2".to_string()
                            ))]),
                            true
                        ),
                        BmsCaseBlock(
                            SwitchLabel::Case(4),
                            BmsBlock(vec![BmsElement::Command(Title(
                                "タイトル4".to_string()
                            ))]),
                            true
                        ),
                        BmsCaseBlock(
                            SwitchLabel::Default,
                            BmsBlock(vec![
                                BmsElement::Command(Title(
                                    "タイトル*".to_string()
                                )),
                                BmsElement::Switch(BmsSwitchBlock(
                                    RandomValue::Max(100),
                                    vec![],
                                    vec![].into_iter().collect()
                                )),
                            ]),
                            false
                        ),
                    ],
                    vec![1, 2, 4].into_iter().collect()
                )),
                BmsElement::Switch(BmsSwitchBlock(
                    RandomValue::Set(123456789012345678901234567890),
                    vec![
                        BmsCaseBlock(
                            SwitchLabel::Case(10),
                            BmsBlock(vec![]),
                            true
                        ),
                        BmsCaseBlock(
                            SwitchLabel::Default,
                            BmsBlock(vec![]),
                            false
                        ),
                        BmsCaseBlock(
                            SwitchLabel::Case(20),
                            BmsBlock(vec![]),
                            false
                        ),
                    ],
                    vec![10, 20].into_iter().collect()
                )),
            ])
        );
    }

    //#[test]
    fn nest_test() {
        use token::{
            ControlFlow::{
                Case, EndIf, EndRandom, EndSwitch, If, Random, Switch,
            },
            Token::ControlFlow,
        };
        let mut token_stream = vec![];
        for i in 0..100 {
            token_stream.push(ControlFlow(Random(i)));
            token_stream.push(ControlFlow(If(i)));
            token_stream.push(ControlFlow(Switch(i)));
            token_stream.push(ControlFlow(Case(i)));
        }
        for _ in 0..100 {
            token_stream.push(ControlFlow(EndSwitch));
            token_stream.push(ControlFlow(EndIf));
            token_stream.push(ControlFlow(EndRandom));
        }
        let b = block.parse_next(&mut token_stream.as_slice()).unwrap();
        println!("{b:?}");
    }
}
