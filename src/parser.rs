use combine::{choice, token, Parser, many1, parser, try};
use combine::char::{digit, spaces, string_cmp};
use combine::primitives::{Stream, ParseResult};
use combine::combinator::FnParser;

use brdgme_game::parser::cmp_ignore_case;
use card::{Expedition, Value, Card};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Play(Card),
    Discard(Card),
    Take(Expedition),
    Draw,
}

type FnP<T, I> = FnParser<I, fn(I) -> ParseResult<T, I>>;

pub fn command<I>() -> FnP<Command, I>
    where I: Stream<Item = char>
{
    fn command_<I>(input: I) -> ParseResult<Command, I>
        where I: Stream<Item = char>
    {
        choice([play, discard, take, draw]).parse_stream(input)
    }
    parser(command_)
}

fn play<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string_cmp("play", cmp_ignore_case)), spaces(), parser(card))
        .map(|(_, _, c)| Command::Play(c))
        .parse_stream(input)
}

fn discard<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string_cmp("discard", cmp_ignore_case)), spaces(), parser(card))
        .map(|(_, _, c)| Command::Discard(c))
        .parse_stream(input)
}

fn take<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string_cmp("take", cmp_ignore_case)),
     spaces(),
     parser(expedition).message("Expected expedition, eg. 'W' or 'Y'"))
            .map(|(_, _, e)| Command::Take(e))
            .parse_stream(input)
}

fn draw<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    try(string_cmp("draw", cmp_ignore_case)).map(|_| Command::Draw).parse_stream(input)
}

fn card<I>(input: I) -> ParseResult<Card, I>
    where I: Stream<Item = char>
{
    (parser(expedition), parser(value))
        .message("Expected card, eg. 'WX' or 'R7'")
        .parse_stream(input)
}

fn expedition<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([expedition_red, expedition_green, expedition_white, expedition_blue, expedition_yellow])
        .parse_stream(input)
}

fn expedition_red<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('r'), token('R')]).expected('R').map(|_| Expedition::Red).parse_stream(input)
}

fn expedition_green<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('g'), token('G')]).expected('G').map(|_| Expedition::Green).parse_stream(input)
}

fn expedition_white<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('w'), token('W')]).expected('W').map(|_| Expedition::White).parse_stream(input)
}

fn expedition_blue<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('b'), token('B')]).expected('B').map(|_| Expedition::Blue).parse_stream(input)
}

fn expedition_yellow<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('y'), token('Y')]).expected('Y').map(|_| Expedition::Yellow).parse_stream(input)
}

fn value<I>(input: I) -> ParseResult<Value, I>
    where I: Stream<Item = char>
{
    choice([value_investment, value_n]).parse_stream(input)
}

fn value_investment<I>(input: I) -> ParseResult<Value, I>
    where I: Stream<Item = char>
{
    choice([token('x'), token('X')]).expected('X').map(|_| Value::Investment).parse_stream(input)
}

fn value_n<I>(input: I) -> ParseResult<Value, I>
    where I: Stream<Item = char>
{
    many1(digit()).map(|d: String| Value::N(d.parse::<usize>().unwrap())).parse_stream(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use combine::Parser;
    use card::{Expedition, Value};

    #[test]
    fn command_works() {
        assert_eq!(command().parse("play y8"),
                   Ok((Command::Play((Expedition::Yellow, Value::N(8))), "")));
        assert_eq!(command().parse("discard bx"),
                   Ok((Command::Discard((Expedition::Blue, Value::Investment)), "")));
        assert_eq!(command().parse("take r"),
                   Ok((Command::Take(Expedition::Red), "")));
        assert_eq!(command().parse("draw"), Ok((Command::Draw, "")));
    }
}
