use ::Game;
use card::{Expedition, Card};
use brdgme_game::{Commander, Log};
use brdgme_game::error::GameError;
use parser;
use nom::IResult::*;

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
        match parser::command(input) {
            Done(_, Command::Play(c)) => self.play(player, c),
            Done(_, Command::Discard(c)) => self.discard(player, c),
            Done(_, Command::Take(e)) => self.take(player, e),
            Done(_, Command::Draw) => self.draw(player),
            _ => Err(GameError::InvalidInput("nope".to_string())),
        }
    }
}
