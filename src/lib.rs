extern crate rand;
#[macro_use]
extern crate nom;
extern crate brdgme;

mod card;
mod parser;
mod command;

use rand::{thread_rng, Rng};
use brdgme::{Gamer, Log};
use brdgme::error::GameError;

const INVESTMENTS: usize = 3;
const ROUNDS: usize = 3;
const START_ROUND: usize = 1;
const PLAYERS: usize = 2;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;
const HAND_SIZE: usize = 8;

pub enum Phase {
    PlayOrDiscard,
    DrawOrTake,
}

impl Default for Phase {
    fn default() -> Phase {
        Phase::PlayOrDiscard
    }
}

#[derive(Default)]
pub struct Game {
    pub round: usize,
    pub phase: Phase,
    pub deck: card::Deck,
    pub discards: card::Deck,
    pub hands: Vec<card::Deck>,
    pub expeditions: Vec<card::Deck>,
    pub current_player: usize,
}

fn initial_deck() -> Vec<card::Card> {
    let mut deck: card::Deck = vec![];
    for e in card::expeditions() {
        for _ in 0..INVESTMENTS {
            deck.push(card::Card {
                expedition: e,
                value: card::Value::Investment,
            });
        }
        for v in MIN_VALUE..MAX_VALUE + 1 {
            deck.push(card::Card {
                expedition: e,
                value: card::Value::N(v),
            });
        }
    }
    return deck;
}

impl Game {
    pub fn new() -> Game {
        Game::default()
    }

    fn start_round(&mut self) -> Result<Vec<Log>, GameError> {
        let mut deck = initial_deck();
        let mut logs: Vec<Log> = vec![
            Log::public(format!("Starting round {}", self.round)),
        ];
        thread_rng().shuffle(deck.as_mut_slice());
        self.deck = deck;
        self.discards = vec![];
        self.hands = vec![];
        self.expeditions = vec![];
        for p in 0..PLAYERS {
            self.hands.push(vec![]);
            self.expeditions.push(vec![]);
            logs.extend(try!(self.draw(p)));
        }
        Ok(logs)
    }

    fn next_round(&mut self) -> Result<Vec<Log>, GameError> {
        if self.round < ROUNDS {
            self.round += 1;
            self.start_round()
        } else {
            // TODO end game log
            Ok(vec![])
        }
    }

    pub fn draw_from_deck(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_players_turn(player));
        self.draw(player)
    }

    fn draw(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        match self.hands.get_mut(player) {
            Some(hand) => {
                let mut num = HAND_SIZE - hand.len();
                let dl = self.deck.len();
                if num > dl {
                    num = dl;
                }
                for c in self.deck.drain(..num) {
                    hand.push(c);
                }
            }
            None => return Err(GameError::Internal("Invalid player number".to_string())),
        };
        if self.deck.len() == 0 {
            self.next_round()
        } else {
            Ok(vec![])
        }
    }
}

impl Gamer for Game {
    fn start(&mut self, players: usize) -> Result<Vec<Log>, GameError> {
        if players != PLAYERS {
            return Err(GameError::PlayerCount(2, 2, players));
        }
        self.round = START_ROUND;
        self.start_round()
    }

    fn is_finished(&self) -> bool {
        self.round >= ROUNDS
    }

    fn whose_turn(&self) -> Vec<usize> {
        vec![self.current_player]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use brdgme::Gamer;

    #[test]
    fn start_works() {
        let mut game = Game::new();
        assert!(game.start(2).is_ok());
        assert_eq!(game.hands.len(), 2);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }
}
