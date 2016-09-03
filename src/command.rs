use ::Game;
use card::{Expedition, Card};
use brdgme_game::{Commander, Log};
use brdgme_game::error::GameError;
use combine::Parser;
use parser;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Play(Card),
    Discard(Card),
    Take(Expedition),
    Draw,
}

impl Commander for Game {
    fn command(&mut self,
               player: usize,
               input: &str,
               _players: &[String])
               -> Result<(Vec<Log>, String), GameError> {
        match parser::command().parse(input) {
            Ok((Command::Play(c), remaining)) => {
                self.play(player, c).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Discard(c), remaining)) => {
                self.discard(player, c).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Take(e), remaining)) => {
                self.take(player, e).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Draw, remaining)) => self.draw(player).map(|l| (l, remaining.to_string())),
            _ => Err(GameError::InvalidInput("nope".to_string())),
        }
    }
}
