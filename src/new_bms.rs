pub(crate) mod lex;
pub(crate) mod token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawBms(Vec<token::Token>);

impl RawBms {
    pub fn parse(source: &str) -> RawBms {
        let token_stream = source
            .lines()
            .enumerate()
            .filter_map(|(line, input)| {
                lex::lex(input).unwrap_or_else(|err| {
                    //log::warn!("{}行目の解析に失敗しました。", line + 1);
                    //log::warn!("{err}");
                    None
                })
            })
            .collect();
        RawBms(token_stream)
    }
}
