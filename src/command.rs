use ::Game;
use brdgme::{Commander, Log};
use brdgme::error::GameError;

impl Commander for Game {
    fn command(&mut self, player: usize, input: &[u8]) -> Result<(Vec<Log>, &[u8]), GameError> {
        return Ok((vec![], &[]));
    }
}
