use ::Game;
use card::{Expedition, Card};
use brdgme_game::{Commander, Log};
use brdgme_game::error::GameError;
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
               _players: &Vec<String>)
               -> Result<Vec<Log>, GameError> {
        match try!(parser::command(input)) {
            Command::Play(c) => self.play(player, c),
            Command::Discard(c) => self.discard(player, c),
            Command::Take(e) => self.take(player, e),
            Command::Draw => self.draw(player),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use card::{Expedition, Value};
    use parser;

    #[test]
    fn command_works() {
        assert_eq!(parser::command("PLaY y8"),
                   Ok(Command::Play((Expedition::Yellow, Value::N(8)))));
        assert_eq!(parser::command("diSCArd bX"),
                   Ok(Command::Discard((Expedition::Blue, Value::Investment))));
        assert_eq!(parser::command("tAKE R"),
                   Ok(Command::Take(Expedition::Red)));
        assert_eq!(parser::command("dRaW"), Ok(Command::Draw));
    }
}
