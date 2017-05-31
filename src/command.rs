use brdgme_game::command::parser::*;
use brdgme_game::Gamer;

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
    pub fn command_parser(&self, player: usize) -> Option<Box<Parser<Command>>> {
        if self.is_finished() {
            return None;
        }
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
        if parsers.is_empty() {
            None
        } else {
            Some(Box::new(OneOf::new(parsers)))
        }
    }

    pub fn play_parser(&self, player: usize) -> impl Parser<Command> {
        Doc::name_desc("play",
                       "play a card to an expedition",
                       Map::new(Chain2::new(Token::new("play"),
                                            AfterSpace::new(self.player_card_parser(player))),
                                |(_, c)| Command::Play(c)))
    }

    pub fn discard_parser(&self, player: usize) -> impl Parser<Command> {
        Doc::name_desc("discard",
                       "discard a card to the common discard piles",
                       Map::new(Chain2::new(Token::new("discard"),
                                            AfterSpace::new(self.player_card_parser(player))),
                                |(_, c)| Command::Discard(c)))
    }

    pub fn player_card_parser(&self, player: usize) -> impl Parser<Card> {
        let mut player_hand = self.hands.get(player).cloned().unwrap_or_else(|| vec![]);
        player_hand.sort();
        player_hand.dedup();
        Doc::name_desc("card", "the card in your hand", Enum::exact(player_hand))
    }
}

pub fn draw_parser() -> impl Parser<Command> {
    Doc::name_desc("draw",
                   "draw a card from the draw pile",
                   Map::new(Token::new("draw"), |_| Command::Draw))
}

pub fn take_parser() -> impl Parser<Command> {
    Doc::name_desc("take",
                   "take the top card from one of the common discard piles",
                   Map::new(Chain2::new(Token::new("take"), AfterSpace::new(expedition_parser())),
                            |(_, e)| Command::Take(e)))
}

pub fn expedition_parser() -> impl Parser<Expedition> {
    Doc::name("expedition", Enum::exact(expeditions()))
}
