use ::Game;
use brdgme_game::{Commander, Log};
use brdgme_game::error::GameError;

impl Commander for Game {
    fn command(&mut self, _player: usize, _input: &[u8]) -> Result<(Vec<Log>, &[u8]), GameError> {
        return Ok((vec![], &[]));
    }
}
