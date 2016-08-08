extern crate rand;
#[macro_use]
extern crate nom;
extern crate rustc_serialize;
extern crate brdgme_game;
extern crate brdgme_color;

mod card;
mod command;

use card::{Card, Expedition, Value, Deck};
use rand::{thread_rng, Rng};
use brdgme_game::{Gamer, Log};
use brdgme_game::error::GameError;

const INVESTMENTS: usize = 3;
const ROUNDS: usize = 3;
pub const START_ROUND: usize = 1;
const PLAYERS: usize = 2;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;
const HAND_SIZE: usize = 8;

#[derive(PartialEq, Copy, Clone, RustcDecodable, RustcEncodable, Debug)]
pub enum Phase {
    PlayOrDiscard,
    DrawOrTake,
}

impl Default for Phase {
    fn default() -> Phase {
        Phase::PlayOrDiscard
    }
}

#[derive(Default, RustcDecodable, RustcEncodable, PartialEq, Debug)]
pub struct Game {
    pub round: usize,
    pub phase: Phase,
    pub deck: Deck,
    pub discards: Deck,
    pub hands: Vec<Deck>,
    pub scores: Vec<Vec<usize>>,
    pub expeditions: Vec<Deck>,
    pub current_player: usize,
}

fn initial_deck() -> Vec<Card> {
    let mut deck: Deck = vec![];
    for e in card::expeditions() {
        for _ in 0..INVESTMENTS {
            deck.push(Card {
                expedition: e,
                value: Value::Investment,
            });
        }
        for v in MIN_VALUE..MAX_VALUE + 1 {
            deck.push(Card {
                expedition: e,
                value: Value::N(v),
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
        let mut logs: Vec<Log> = vec![
            Log::public(format!("Starting round {}", self.round)),
        ];
        // Grab a new deck and shuffle it.
        let mut deck = initial_deck();
        thread_rng().shuffle(deck.as_mut_slice());
        self.deck = deck;
        // Clear out discards, hands and expeditions.
        self.discards = vec![];
        self.hands = vec![];
        self.expeditions = vec![];
        // Initialise player hands and expedition and draw initial cards.
        for p in 0..PLAYERS {
            self.hands.push(vec![]);
            self.expeditions.push(vec![]);
            logs.extend(try!(self.draw_hand_full(p)));
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

    fn assert_phase(&self, phase: Phase) -> Result<(), GameError> {
        if phase == self.phase {
            Ok(())
        } else {
            Err(GameError::InvalidInput("not the right phase".to_string()))
        }
    }

    pub fn draw(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::DrawOrTake));
        let logs = try!(self.draw_hand_full(player));
        self.next_phase();
        Ok(logs)
    }

    fn next_phase(&mut self) {
        match self.phase {
            Phase::PlayOrDiscard => {
                self.phase = Phase::DrawOrTake;
            }
            Phase::DrawOrTake => {
                self.next_player();
            }
        }
    }

    fn next_player(&mut self) {
        self.phase = Phase::PlayOrDiscard;
        self.current_player = (self.current_player + 1) % 2;
    }

    pub fn take(&mut self, player: usize, expedition: Expedition) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::DrawOrTake));
        if let Some(index) = self.discards.iter().rposition(|ref c| c.expedition == expedition) {
            let c = *try!(self.discards
                .get(index)
                .ok_or(GameError::Internal("could not find discard card".to_string())));
            try!(self.hands
                    .get_mut(player)
                    .ok_or(GameError::Internal("could not find player hand".to_string())))
                .push(c);
            self.discards.remove(index);
            self.next_phase();
            Ok(vec![
                Log::public(format!("{{player {}}} took {}", player, c).to_string()),
            ])
        } else {
            Err(GameError::InvalidInput("there are no discarded cards for that expedition"
                .to_string()))
        }
    }

    pub fn remove_player_card(&mut self, player: usize, c: Card) -> Result<(), GameError> {
        try!(self.hands
            .get_mut(player)
            .ok_or(GameError::Internal(format!("could not find player hand for player {}",
                                               player)))
            .and_then(|h| {
                let index = try!(h.iter()
                    .position(|hc| c == *hc)
                    .ok_or(GameError::InvalidInput(format!("you don't have {}", c))));
                h.remove(index);
                Ok(())
            }));
        Ok(())
    }

    pub fn discard(&mut self, player: usize, c: Card) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::PlayOrDiscard));
        try!(self.remove_player_card(player, c));
        self.discards.push(c);
        self.next_phase();
        Ok(vec![])
    }

    fn assert_has_card(&self, player: usize, c: Card) -> Result<(), GameError> {
        try!(self.hands
            .get(player)
            .ok_or(GameError::Internal(format!("could not find player hand for player {}",
                                               player)))
            .and_then(|h| {
                h.iter()
                    .position(|hc| c == *hc)
                    .ok_or(GameError::InvalidInput(format!("you don't have {}", c)))
            }));
        Ok(())
    }

    fn highest_value_in_expedition(&self, player: usize, expedition: Expedition) -> Option<usize> {
        self.expeditions.get(player).and_then(|e| {
            e.iter()
                .filter(|c| c.expedition == expedition && c.value != Value::Investment)
                .map(|c| if let Value::N(n) = c.value {
                    n
                } else {
                    0
                })
                .max()
        })
    }

    pub fn play(&mut self, player: usize, c: Card) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        try!(self.assert_phase(Phase::PlayOrDiscard));
        try!(self.assert_has_card(player, c));
        if let Some(hn) = self.highest_value_in_expedition(player, c.expedition) {
            match c.value {
                Value::Investment => {
                    return Err(GameError::InvalidInput(format!("you can't play {} as you've \
                                                                already played a higher card",
                                                               c)));
                }
                Value::N(n) => {
                    if n <= hn {
                        return Err(GameError::InvalidInput(format!("you can't play {} as \
                                                                    you've already played a \
                                                                    higher card",
                                                                   c)));
                    }
                }
            }
        }
        try!(self.remove_player_card(player, c));
        try!(self.expeditions
                .get_mut(player)
                .ok_or(GameError::Internal(format!("could not find player expedition for \
                                                    player {}",
                                                   player))))
            .push(c);
        self.next_phase();
        Ok(vec![])
    }

    fn draw_hand_full(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
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
            None => return Err(GameError::Internal("invalid player number".to_string())),
        };
        if self.deck.len() == 0 {
            self.next_round()
        } else {
            Ok(vec![])
        }
    }

    fn player_score(&self, player: usize) -> usize {
        self.scores.iter().fold(0, |acc, rs| acc + rs.get(player).unwrap_or(&0))
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

    fn winners(&self) -> Vec<usize> {
        if !self.is_finished() {
            return vec![];
        }
        let p0_score = self.player_score(0);
        let p1_score = self.player_score(1);
        if p0_score > p1_score {
            vec![0]
        } else if p1_score > p0_score {
            vec![1]
        } else {
            vec![0, 1]
        }
    }

    fn whose_turn(&self) -> Vec<usize> {
        vec![self.current_player]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::card::{Card, Expedition, Value};
    use brdgme_game::Gamer;

    fn discard_and_draw(game: &mut Game, player: usize) {
        let c = game.hands[player][0];
        game.discard(player, c).unwrap();
        game.draw(player).unwrap();
    }

    #[test]
    fn start_works() {
        let mut game = Game::new();
        game.start(2).unwrap();
        assert_eq!(game.hands.len(), 2);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }

    #[test]
    fn next_round_works() {
        let mut game = Game::new();
        game.start(2).unwrap();
        for _ in 0..22 {
            let c = game.hands[0][0];
            game.discard(0, c).unwrap();
            game.draw(0).unwrap();
            let c = game.hands[1][0];
            game.discard(1, c).unwrap();
            assert_eq!(START_ROUND, game.round);
            game.draw(1).unwrap();
        }
        assert_eq!(START_ROUND + 1, game.round);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }

    #[test]
    fn play_works() {
        let mut game = Game::new();
        game.start(2).unwrap();
        game.hands[0] = vec![
            Card{expedition: Expedition::Green, value: Value::Investment},
            Card{expedition: Expedition::Green, value: Value::Investment},
            Card{expedition: Expedition::Green, value: Value::N(2)},
            Card{expedition: Expedition::Green, value: Value::N(3)},
            Card{expedition: Expedition::Yellow, value: Value::Investment},
            Card{expedition: Expedition::Yellow, value: Value::Investment},
            Card{expedition: Expedition::Yellow, value: Value::N(2)},
            Card{expedition: Expedition::Yellow, value: Value::N(3)},
        ];
        game.play(0,
                  Card {
                      expedition: Expedition::Green,
                      value: Value::Investment,
                  })
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        game.play(0,
                  Card {
                      expedition: Expedition::Green,
                      value: Value::N(2),
                  })
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        // Shouldn't be able to play GX now.
        assert!(game.play(0,
                  Card {
                      expedition: Expedition::Green,
                      value: Value::Investment,
                  })
            .is_err());
        game.play(0,
                  Card {
                      expedition: Expedition::Green,
                      value: Value::N(3),
                  })
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        game.play(0,
                  Card {
                      expedition: Expedition::Yellow,
                      value: Value::N(3),
                  })
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        // Shouldn't be able to play Y2 now.
        assert!(game.play(0,
                  Card {
                      expedition: Expedition::Yellow,
                      value: Value::N(2),
                  })
            .is_err());
    }

    #[test]
    fn encode_and_decode_works() {
        use rustc_serialize::json;

        let mut game = Game::new();
        game.start(2).unwrap();

        let encoded = json::encode(&game).unwrap();
        let decoded: Game = json::decode(&encoded).unwrap();
        assert_eq!(game, decoded);
    }
}
