use brdgme_game::{Renderer, GameError};
use brdgme_markup::ast::Node;
use super::Game;

impl Renderer for Game {
    fn render(&self, _player: Option<usize>) -> Result<Vec<Node>, GameError> {
        Err(GameError::Internal("not implemented".to_string()))
    }
}
