use brdgme_game::command::parser::*;

use card::{Card, Expedition, expeditions};
use Game;
use Phase;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Play(Card),
    Discard(Card),
    Take(Expedition),
    Draw,
}

impl Game {
    pub fn command_parser(&self, player: usize) -> Box<Parser<Command>> {
        let mut parsers: Vec<Box<Parser<Command>>> = vec![];
        if self.current_player == player {
            match self.phase {
                Phase::PlayOrDiscard => {
                    parsers.push(Box::new(self.play_parser(player)));
                    parsers.push(Box::new(self.discard_parser(player)));
                }
                Phase::DrawOrTake => {
                    parsers.push(Box::new(draw_parser()));
                    parsers.push(Box::new(take_parser()));
                }
            }
        }
        Box::new(OneOf::new(parsers))
    }

    pub fn play_parser(&self, player: usize) -> impl Parser<Command> {
        Map::new(Chain2::new(Token::new("play"), self.player_card_parser(player)),
                 |(_, c)| Command::Play(c))
    }

    pub fn discard_parser(&self, player: usize) -> impl Parser<Command> {
        Map::new(Chain2::new(Token::new("discard"), self.player_card_parser(player)),
                 |(_, c)| Command::Discard(c))
    }

    pub fn player_card_parser(&self, player: usize) -> impl Parser<Card> {
        Enum::exact(self.hands
                        .get(player)
                        .cloned()
                        .unwrap_or_else(|| vec![]))
    }
}

pub fn draw_parser() -> impl Parser<Command> {
    Map::new(Token::new("draw"), |_| Command::Draw)
}

pub fn take_parser() -> impl Parser<Command> {
    Map::new(Chain2::new(Token::new("take"), expedition_parser()),
             |(_, e)| Command::Take(e))
}

pub fn expedition_parser() -> impl Parser<Expedition> {
    Enum::exact(expeditions())
}
