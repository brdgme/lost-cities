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
               -> Result<Vec<Log>, GameError> {
        match parser::command().parse(input) {
            Ok((Command::Play(c), _)) => self.play(player, c),
            Ok((Command::Discard(c), _)) => self.discard(player, c),
            Ok((Command::Take(e), _)) => self.take(player, e),
            Ok((Command::Draw, _)) => self.draw(player),
            _ => Err(GameError::InvalidInput("nope".to_string())),
        }
    }
}
