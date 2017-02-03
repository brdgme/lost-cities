extern crate rand;
extern crate combine;
#[macro_use]
extern crate serde_derive;

extern crate brdgme_game;
extern crate brdgme_color;
extern crate brdgme_markup;

mod card;
mod render;
mod parser;

use std::collections::HashMap;

use combine::Parser;

use card::{Card, Expedition, Value, expeditions};
use rand::{thread_rng, Rng};
use parser::Command;
use brdgme_game::{Gamer, Log};
use brdgme_game::error::GameError;
use brdgme_markup::ast::Node as N;

const INVESTMENTS: usize = 3;
pub const ROUNDS: usize = 3;
pub const START_ROUND: usize = 1;
const PLAYERS: usize = 2;
const MIN_VALUE: usize = 2;
const MAX_VALUE: usize = 10;
const HAND_SIZE: usize = 8;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Phase {
    PlayOrDiscard,
    DrawOrTake,
}

impl Default for Phase {
    fn default() -> Phase {
        Phase::PlayOrDiscard
    }
}

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub round: usize,
    pub phase: Phase,
    pub deck: Vec<Card>,
    pub discards: Vec<Card>,
    pub hands: Vec<Vec<Card>>,
    pub scores: Vec<Vec<isize>>,
    pub expeditions: Vec<Vec<Card>>,
    pub current_player: usize,
    pub discarded_expedition: Option<Expedition>,
}

#[derive(Default, Serialize)]
pub struct PubState {
    pub player: Option<usize>,
    pub round: usize,
    pub is_finished: bool,
    pub phase: Phase,
    pub deck_remaining: usize,
    pub discards: HashMap<Expedition, Value>,
    pub hand: Option<Vec<Card>>,
    pub scores: Vec<Vec<isize>>,
    pub expeditions: Vec<Vec<Card>>,
    pub current_player: usize,
}

fn initial_deck() -> Vec<Card> {
    let mut deck: Vec<Card> = vec![];
    for e in card::expeditions() {
        for _ in 0..INVESTMENTS {
            deck.push((e, Value::Investment));
        }
        for v in MIN_VALUE..MAX_VALUE + 1 {
            deck.push((e, Value::N(v)));
        }
    }
    deck
}

impl Game {
    fn start_round(&mut self) -> Result<Vec<Log>, GameError> {
        let mut logs: Vec<Log> = vec![Log::public(vec![N::text(format!("Starting round {}",
                                                                       self.round))])];
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
            logs.extend(self.draw_hand_full(p)?);
        }
        if self.round > START_ROUND {
            // Player with the most points starts next, otherwise the next player.
            self.current_player = match self.player_score(0) - self.player_score(1) {
                0 => opponent(self.current_player),
                s if s > 0 => 0,
                _ => 1,
            }
        }
        self.start_turn();
        Ok(logs)
    }

    fn end_round(&mut self) -> Result<Vec<Log>, GameError> {
        self.round += 1;
        let mut logs: Vec<Log> = vec![];
        for p in 0..2 {
            let mut round_score: isize = 0;
            if let Some(p_exp) = self.expeditions.get(p) {
                round_score = score(p_exp);
            }
            self.scores.get_mut(p).map(|s| s.push(round_score));
            logs.push(Log::public(vec![N::Player(p),
                                       N::text(" scored "),
                                       N::Bold(vec![N::text(format!("{}", round_score))]),
                                       N::text(" points, now on "),
                                       N::Bold(vec![N::text(format!("{}",
                                                                    self.player_score(p)))])]));
        }
        if self.round < START_ROUND + ROUNDS {
            self.start_round().map(|l| {
                logs.extend(l);
                logs
            })
        } else {
            logs.push(self.game_over_log());
            Ok(logs)
        }
    }

    fn game_over_log(&self) -> Log {
        let scores: [isize; 2] = [self.player_score(0), self.player_score(1)];
        let winners = self.winners();
        Log::public(vec![N::Bold(vec![N::text("The game is over, "),
                                      match winners.as_slice() {
                                          w if w.len() == 1 => {
                                              let p = w[0];
                                              N::Group(vec![
                    N::Player(p),
                    N::text(format!(" won by {} points", scores.get(p).unwrap_or(&0)-
                    scores.get(opponent(p)).unwrap_or(&0))),
                ])
                                          }
                                          _ => {
                                              N::text(format!("scores tied at {}",
                                                              scores.first().unwrap_or(&0)))
                                          }
                                      }])])
    }

    fn assert_phase(&self, phase: Phase) -> Result<(), GameError> {
        if phase == self.phase {
            Ok(())
        } else {
            Err(GameError::InvalidInput("Not the right phase".to_string()))
        }
    }

    pub fn draw(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        self.assert_phase(Phase::DrawOrTake)?;
        let r = self.round;
        let logs = self.draw_hand_full(player)?;
        if r == self.round {
            // Only run next phase if a new round wasn't started, if a new round
            // was started then everything will already be initialised.
            self.next_phase();
        }
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
        self.current_player = (self.current_player + 1) % 2;
        self.start_turn();
    }

    fn start_turn(&mut self) {
        self.phase = Phase::PlayOrDiscard;
        self.discarded_expedition = None;
    }

    pub fn take(&mut self, player: usize, expedition: Expedition) -> Result<Vec<Log>, GameError> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        self.assert_phase(Phase::DrawOrTake)?;
        if self.discarded_expedition == Some(expedition) {
            return Err(GameError::InvalidInput("You can't take the same card you just discarded"
                .to_string()));
        }
        if let Some(index) = self.discards.iter().rposition(|&(e, _)| e == expedition) {
            let c = *self.discards
                .get(index)
                .ok_or_else(|| GameError::Internal("could not find discard card".to_string()))?;
            self.hands
                .get_mut(player)
                .ok_or_else(|| GameError::Internal("could not find player hand".to_string()))?
                .push(c);
            self.discards.remove(index);
            self.next_phase();
            Ok(vec![Log::public(vec![N::Player(player), N::text(" took "), render::card(&c)])])
        } else {
            Err(GameError::InvalidInput("There are no discarded cards for that expedition"
                .to_string()))
        }
    }

    pub fn available_discard(&self, expedition: Expedition) -> Option<Card> {
        self.discards.iter().rev().find(|c| c.0 == expedition).cloned()
    }

    pub fn remove_player_card(&mut self, player: usize, c: Card) -> Result<(), GameError> {
        self.hands
            .get_mut(player)
            .ok_or_else(|| {
                GameError::Internal(format!("could not find player hand for player {}", player))
            })
            .and_then(|h| {
                let index = h.iter()
                    .position(|hc| c == *hc)
                    .ok_or_else(|| {
                        GameError::InvalidInput(format!("You don't have {}", render::card_text(&c)))
                    })?;
                h.remove(index);
                Ok(())
            })?;
        Ok(())
    }

    pub fn discard(&mut self, player: usize, c: Card) -> Result<Vec<Log>, GameError> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        self.assert_phase(Phase::PlayOrDiscard)?;
        self.remove_player_card(player, c)?;
        self.discards.push(c);
        let (e, _) = c;
        self.discarded_expedition = Some(e);
        self.next_phase();
        Ok(vec![Log::public(vec![N::Player(player), N::text(" discarded "), render::card(&c)])])
    }

    fn assert_has_card(&self, player: usize, c: Card) -> Result<(), GameError> {
        self.hands
            .get(player)
            .ok_or_else(|| {
                GameError::Internal(format!("could not find player hand for player {}", player))
            })
            .and_then(|h| {
                h.iter()
                    .position(|hc| c == *hc)
                    .ok_or_else(|| {
                        GameError::InvalidInput(format!("You don't have {}", render::card_text(&c)))
                    })
            })?;
        Ok(())
    }

    fn highest_value_in_expedition(&self, player: usize, expedition: Expedition) -> Option<usize> {
        self.expeditions.get(player).and_then(|e| {
            e.iter()
                .filter(|&c| c.0 == expedition && c.1 != Value::Investment)
                .map(|&c| if let Value::N(n) = c.1 { n } else { 0 })
                .max()
        })
    }

    pub fn play(&mut self, player: usize, c: Card) -> Result<Vec<Log>, GameError> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        self.assert_phase(Phase::PlayOrDiscard)?;
        self.assert_has_card(player, c)?;
        let (e, v) = c;
        if let Some(hn) = self.highest_value_in_expedition(player, e) {
            match v {
                Value::Investment => {
                    return Err(GameError::InvalidInput(format!("You can't play {} as you've \
                                                                already played a higher card",
                                                               render::card_text(&c))));
                }
                Value::N(n) => {
                    if n <= hn {
                        return Err(GameError::InvalidInput(format!("You can't play {} as \
                                                                    you've already played a \
                                                                    higher card",
                                                                   render::card_text(&c))));
                    }
                }
            }
        }
        self.remove_player_card(player, c)?;
        self.expeditions
            .get_mut(player)
            .ok_or_else(|| {
                GameError::Internal(format!("could not find player expedition for player {}",
                                            player))
            })?
            .push(c);
        self.next_phase();
        Ok(vec![Log::public(vec![N::Player(player), N::text(" played "), render::card(&c)])])
    }

    fn draw_hand_full(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        let mut logs: Vec<Log> = vec![];
        match self.hands.get_mut(player) {
            Some(hand) => {
                let mut num = HAND_SIZE - hand.len();
                let dl = self.deck.len();
                if num > dl {
                    num = dl;
                }
                let mut drawn: Vec<Card> = vec![];
                for c in self.deck.drain(..num) {
                    hand.push(c);
                    drawn.push(c);
                }
                drawn.sort();
                let d_len = drawn.len();
                let mut public_log: Vec<N> = vec![N::Player(player), N::text(" drew ")];
                if d_len == 1 {
                    public_log.append(&mut vec![N::text("a card")]);
                } else {
                    public_log.append(&mut vec![N::Bold(vec![N::text(format!("{}",
                                                                             drawn.len()))]),
                                                N::text(" cards")]);
                }
                public_log.append(&mut vec![N::text(", "),
                                            N::Bold(vec![N::text(format!("{}",
                                                                         self.deck.len()))]),
                                            N::text(" remaining")]);
                logs.push(Log::public(public_log));
                let mut private_log: Vec<N> = vec![N::text("You drew ")];
                private_log.append(&mut render::comma_cards(&drawn));
                logs.push(Log::private(private_log, vec![player]));
            }
            None => return Err(GameError::Internal("invalid player number".to_string())),
        };
        if self.deck.is_empty() {
            self.end_round()
        } else {
            Ok(logs)
        }
    }

    fn player_score(&self, player: usize) -> isize {
        match self.scores.get(player) {
            Some(s) => s.iter().sum(),
            None => 0,
        }
    }
}

pub fn opponent(player: usize) -> usize {
    (player + 1) % 2
}

impl Gamer for Game {
    type PubState = PubState;

    fn start(&mut self, players: usize) -> Result<Vec<Log>, GameError> {
        if players != PLAYERS {
            return Err(GameError::PlayerCount(2, 2, players));
        }
        self.round = START_ROUND;
        self.scores = vec![vec![], vec![]];
        self.start_round()
    }

    fn is_finished(&self) -> bool {
        self.round >= START_ROUND + ROUNDS
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

    fn pub_state(&self, player: Option<usize>) -> Self::PubState {
        PubState {
            player: match player {
                Some(p) if p < 2 => Some(p),
                _ => None,
            },
            round: self.round,
            is_finished: self.is_finished(),
            phase: self.phase,
            deck_remaining: self.deck.len(),
            discards: {
                let mut d: HashMap<Expedition, Value> = HashMap::new();
                for e in card::expeditions() {
                    if let Some(c) = card::last_expedition(&self.discards, e) {
                        d.insert(e, c.1);
                    }
                }
                d
            },
            hand: player.and_then(|p| self.hands.get(p).cloned()),
            scores: self.scores.clone(),
            expeditions: self.expeditions.clone(),
            current_player: self.current_player,
        }
    }

    fn command(&mut self,
               player: usize,
               input: &str,
               _players: &[String])
               -> Result<(Vec<Log>, String), GameError> {
        match parser::command().parse(input) {
            Ok((Command::Play(c), remaining)) => {
                self.play(player, c).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Discard(c), remaining)) => {
                self.discard(player, c).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Take(e), remaining)) => {
                self.take(player, e).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Draw, remaining)) => self.draw(player).map(|l| (l, remaining.to_string())),
            Err(e) => Err(brdgme_game::parser::to_game_error(e)),
        }
    }
}

pub fn score(cards: &[Card]) -> isize {
    let mut exp_cards: HashMap<Expedition, isize> = HashMap::new();
    let mut exp_inv: HashMap<Expedition, isize> = HashMap::new();
    let mut exp_sum: HashMap<Expedition, isize> = HashMap::new();
    for c in cards {
        let cards = exp_cards.entry(c.0).or_insert(0);
        *cards += 1;
        match c.1 {
            Value::Investment => {
                let inv = exp_inv.entry(c.0).or_insert(0);
                *inv += 1;
            }
            Value::N(n) => {
                let sum = exp_sum.entry(c.0).or_insert(0);
                *sum += n as isize;
            }
        }
    }
    expeditions().iter().fold(0, |acc, &e| {
        let cards = exp_cards.get(&e);
        if cards == None {
            return acc;
        }
        acc + (exp_sum.get(&e).unwrap_or(&0) - 20) * (exp_inv.get(&e).unwrap_or(&0) + 1) +
        if cards.unwrap() >= &8 { 20 } else { 0 }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use super::card::{Expedition, Value};
    use brdgme_game::Gamer;

    fn discard_and_draw(game: &mut Game, player: usize) {
        let c = game.hands[player][0];
        game.discard(player, c).unwrap();
        game.draw(player).unwrap();
    }

    #[test]
    fn start_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        assert_eq!(game.hands.len(), 2);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }

    #[test]
    fn end_round_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        for _ in 0..44 {
            let p = game.current_player;
            let c = game.hands[p][0];
            game.discard(p, c).unwrap();
            assert_eq!(START_ROUND, game.round);
            game.draw(p).unwrap();
        }
        assert_eq!(START_ROUND + 1, game.round);
        assert_eq!(game.hands[0].len(), 8);
        assert_eq!(game.hands[1].len(), 8);
        assert_eq!(game.deck.len(), 44);
    }

    #[test]
    fn game_end_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        for _ in 0..(44 * ROUNDS) {
            let p = game.current_player;
            let c = game.hands[p][0];
            game.discard(p, c).unwrap();
            game.draw(p).unwrap();
        }
        assert_eq!(game.is_finished(), true);
    }

    #[test]
    fn play_works() {
        let mut game = Game::default();
        game.start(2).unwrap();
        game.hands[0] = vec![(Expedition::Green, Value::Investment),
                             (Expedition::Green, Value::Investment),
                             (Expedition::Green, Value::N(2)),
                             (Expedition::Green, Value::N(3)),
                             (Expedition::Yellow, Value::Investment),
                             (Expedition::Yellow, Value::Investment),
                             (Expedition::Yellow, Value::N(2)),
                             (Expedition::Yellow, Value::N(3))];
        game.play(0, (Expedition::Green, Value::Investment))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        game.play(0, (Expedition::Green, Value::N(2)))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        // Shouldn't be able to play GX now.
        assert!(game.play(0, (Expedition::Green, Value::Investment))
            .is_err());
        game.play(0, (Expedition::Green, Value::N(3)))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        game.play(0, (Expedition::Yellow, Value::N(3)))
            .unwrap();
        game.draw(0).unwrap();
        discard_and_draw(&mut game, 1);
        // Shouldn't be able to play Y2 now.
        assert!(game.play(0, (Expedition::Yellow, Value::N(2)))
            .is_err());
    }

    #[test]
    fn score_works() {
        assert_eq!(0, score(&vec![]));
        assert_eq!(-17, score(&vec![(Expedition::Red, Value::N(3))]));
        assert_eq!(-34,
                   score(&vec![(Expedition::Red, Value::N(3)), (Expedition::Green, Value::N(3))]));
        assert_eq!(-30,
                   score(&vec![(Expedition::Red, Value::N(3)),
                               (Expedition::Green, Value::N(3)),
                               (Expedition::Green, Value::N(4))]));
        assert_eq!(-37,
                   score(&vec![(Expedition::Green, Value::Investment),
                               (Expedition::Red, Value::N(3)),
                               (Expedition::Green, Value::N(4)),
                               (Expedition::Green, Value::N(6))]));
        assert_eq!(44,
                   score(&vec![(Expedition::Green, Value::N(2)),
                               (Expedition::Green, Value::N(3)),
                               (Expedition::Green, Value::N(4)),
                               (Expedition::Green, Value::N(5)),
                               (Expedition::Green, Value::N(6)),
                               (Expedition::Green, Value::N(7)),
                               (Expedition::Green, Value::N(8)),
                               (Expedition::Green, Value::N(9))]));
    }
}
