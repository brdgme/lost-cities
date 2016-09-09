use combine::{choice, token, Parser, ParserExt, many1, parser, try};
use combine::char::{digit, string, spaces};
use combine::primitives::{Stream, ParseResult};
use combine::combinator::FnParser;

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
        choice([play, discard, take, draw]).parse_state(input)
    }
    parser(command_)
}

fn play<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string("play")), spaces(), parser(card))
        .map(|(_, _, c)| Command::Play(c))
        .parse_state(input)
}

fn discard<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string("discard")), spaces(), parser(card))
        .map(|(_, _, c)| Command::Discard(c))
        .parse_state(input)
}

fn take<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string("take")),
     spaces(),
     parser(expedition).message("expected expedition, eg. 'w' or 'y'"))
        .map(|(_, _, e)| Command::Take(e))
        .parse_state(input)
}

fn draw<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    try(string("draw"))
        .map(|_| Command::Draw)
        .parse_state(input)
}

fn card<I>(input: I) -> ParseResult<Card, I>
    where I: Stream<Item = char>
{
    (parser(expedition), parser(value))
        .message("expected card, eg. 'wx' or 'r7'")
        .parse_state(input)
}

fn expedition<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([expedition_red, expedition_green, expedition_white, expedition_blue, expedition_yellow])
        .parse_state(input)
}

fn expedition_red<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('r'), token('R')]).map(|_| Expedition::Red).parse_state(input)
}

fn expedition_green<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('g'), token('G')]).map(|_| Expedition::Green).parse_state(input)
}

fn expedition_white<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('w'), token('W')]).map(|_| Expedition::White).parse_state(input)
}

fn expedition_blue<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('b'), token('B')]).map(|_| Expedition::Blue).parse_state(input)
}

fn expedition_yellow<I>(input: I) -> ParseResult<Expedition, I>
    where I: Stream<Item = char>
{
    choice([token('y'), token('Y')]).map(|_| Expedition::Yellow).parse_state(input)
}

fn value<I>(input: I) -> ParseResult<Value, I>
    where I: Stream<Item = char>
{
    choice([value_investment, value_n]).parse_state(input)
}

fn value_investment<I>(input: I) -> ParseResult<Value, I>
    where I: Stream<Item = char>
{
    choice([token('x'), token('X')]).map(|_| Value::Investment).parse_state(input)
}

fn value_n<I>(input: I) -> ParseResult<Value, I>
    where I: Stream<Item = char>
{
    many1(digit()).map(|d: String| Value::N(d.parse::<usize>().unwrap())).parse_state(input)
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
