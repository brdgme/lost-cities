use ::Game;
use card::{Expedition, Card};
use brdgme_game::{Commander, Log};
use brdgme_game::error::GameError;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    Play(Card),
    Discard(Card),
    Take(Expedition),
    Draw,
}

impl Commander for Game {
    fn command(&mut self, _player: usize, _input: &str) -> Result<Vec<Log>, GameError> {
        return Ok(vec![]);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use card::{Card, Expedition, Value};
    use parser;

    #[test]
    fn command_works() {
        assert_eq!(parser::command("PLaY y8"),
                   Ok(Command::Play(Card {
                       expedition: Expedition::Yellow,
                       value: Value::N(8),
                   })));
        assert_eq!(parser::command("diSCArd bX"),
                   Ok(Command::Discard(Card {
                       expedition: Expedition::Blue,
                       value: Value::Investment,
                   })));
        assert_eq!(parser::command("tAKE R"),
                   Ok(Command::Take(Expedition::Red)));
        assert_eq!(parser::command("dRaW"), Ok(Command::Draw));
    }
}
