use brdgme_game::{Renderer, GameError};
use brdgme_markup::ast::{Node as N, Align as A};

use super::Game;
use card::Card;

impl Renderer for Game {
    fn render(&self, player: Option<usize>) -> Result<Vec<N>, GameError> {
        let persp = player.unwrap_or(0);
        if persp > 1 {
            return Err(GameError::Internal("invalid player number".to_string()));
        }
        Ok(vec![N::Align(A::Center,
                         80,
                         vec![
            N::Text("lost cities!".to_string()),
        ])])
    }
}

pub fn card(c: Card) -> N {
    let (e, _) = c;
    N::Bold(vec![N::Fg(e.color(),
                       vec![
                N::Text(card_text(c)),
            ])])
}

pub fn card_text((e, v): Card) -> String {
    format!("{}{}", e, v)
}
