use std::cmp;

use super::{PubState, opponent, START_ROUND, ROUNDS};
use card::{by_expedition, expeditions, Card};

use brdgme_color::GREY;
use brdgme_game::Renderer;
use brdgme_markup::{Node as N, Align as A, Row};

impl Renderer for PubState {
    fn render(&self) -> Vec<N> {
        let mut layout: Vec<Row> = vec![];
        if !self.is_finished {
            layout.extend(vec![vec![(A::Center,
                                     vec![N::text("Round "),
                                          N::Bold(vec![N::text(format!("{}", self.round))]),
                                          N::text(" of "),
                                          N::Bold(vec![N::text(format!("{}", super::ROUNDS))])])],
                               vec![]]);
        }
        layout.push(vec![(A::Center, self.render_tableau())]);
        if let Some(ref h) = self.hand {
            layout.append(&mut vec![vec![],
                                    vec![(A::Center,
                                          vec![N::Fg(GREY.into(), vec![N::text("Your hand")])])],
                                    vec![(A::Center, render_hand(h))]]);
        }
        // Scores
        let persp = match self.player {
            Some(p) if p < 2 => p,
            _ => 0,
        };
        let mut scores: Vec<Row> = vec![];
        let mut header: Row = vec![(A::Left, vec![])];
        for r in START_ROUND..(START_ROUND + ROUNDS) {
            header.extend(vec![(A::Left, vec![N::text("  ")]),
                               (A::Center,
                                vec![N::Fg(GREY.into(), vec![N::text(format!("R{}", r))])])]);
        }
        header.extend(vec![(A::Left, vec![N::text("  ")]),
                           (A::Center, vec![N::Fg(GREY.into(), vec![N::text("Tot")])])]);
        scores.push(header);
        for p in &[persp, opponent(persp)] {
            let mut score_row: Row = vec![(A::Right, vec![N::Player(*p)])];
            for r in 0..ROUNDS {
                score_row.extend(vec![(A::Left, vec![]),
                                      (A::Center,
                                       vec![N::text(self.scores
                                                        .get(*p)
                                                        .and_then(|s| s.get(r))
                                                        .map(|rs| format!("{}", rs))
                                                        .unwrap_or_else(|| {
                                                                            "".to_string()
                                                                        }))])]);
            }
            score_row.extend(vec![(A::Left, vec![]),
                                  (A::Center,
                                   vec![N::text(format!("{}", self.player_score(*p)))])]);
            scores.push(score_row);
        }
        layout.append(&mut vec![vec![],
                                vec![(A::Center,
                                      vec![N::Fg(GREY.into(), vec![N::text("Scores")])])],
                                vec![(A::Center, vec![N::Table(scores)])]]);
        vec![N::Table(layout)]
    }
}

impl PubState {
    fn render_tableau(&self) -> Vec<N> {
        let p = cmp::min(self.player.unwrap_or(0), 1);
        let mut rows: Vec<Row> = vec![];

        // Top half
        let mut top = match self.expeditions.get(super::opponent(p)) {
            Some(e) => render_tableau_cards(e, &N::Player(super::opponent(p))),
            None => vec![],
        };
        top.reverse();
        rows.append(&mut top);

        // Blank row
        rows.push(vec![]);

        // Discards
        let mut discards: Row = vec![(A::Right,
                                      vec![N::Fg(GREY.into(), vec![N::text("Discard ")])])];
        for e in expeditions() {
            // Column spacing
            discards.push((A::Left, vec![N::text("  ")]));

            discards.push((A::Center,
                           vec![if let Some(v) = self.discards.get(&e) {
                                    card(&(e, *v).into())
                                } else {
                                    N::Fg(e.color().into(), vec![N::text("--")])
                                }]));
        }
        discards.push((A::Left,
                       vec![N::Fg(GREY.into(),
                                  vec![N::text("   "),
                                       N::Bold(vec![N::text(format!("{}",
                                                                    self.deck_remaining))]),
                                       N::text(" left")])]));

        rows.push(discards);

        // Blank row
        rows.push(vec![]);

        // Bottom half
        if let Some(e) = self.expeditions.get(p) {
            rows.append(&mut render_tableau_cards(e, &N::Player(p)));
        }
        vec![N::Table(rows)]
    }

    pub fn player_score(&self, player: usize) -> isize {
        match self.scores.get(player) {
            Some(s) => s.iter().sum(),
            None => 0,
        }
    }
}

fn render_tableau_cards(cards: &[Card], header: &N) -> Vec<Row> {
    let mut rows: Vec<Row> = vec![];
    let by_exp = by_expedition(cards);
    let mut largest: usize = 1;
    for e in expeditions() {
        largest = cmp::max(largest, by_exp.get(&e).unwrap_or(&vec![]).len());
    }
    for row_i in 0..largest {
        let mut row: Row = vec![if row_i == 0 {
                                    (A::Right, vec![header.to_owned(), N::text(" ")])
                                } else {
                                    (A::Left, vec![])
                                }];
        for e in expeditions() {
            // Column spacing
            row.push((A::Left, vec![]));
            match by_exp.get(&e).unwrap_or(&vec![]).get(row_i) {
                Some(c) => row.push((A::Center, vec![card(c)])),
                None => row.push((A::Left, vec![])),
            }
        }
        rows.push(row);
    }
    rows
}

fn render_hand(cards: &[Card]) -> Vec<N> {
    let mut output: Vec<N> = vec![];
    let mut sorted = cards.to_owned();
    sorted.sort();
    for c in sorted {
        if !output.is_empty() {
            output.push(N::text(" "));
        }
        output.push(card(&c));
    }
    output
}

pub fn card(c: &Card) -> N {
    N::Bold(vec![N::Fg(c.expedition.color().into(), vec![N::text(c.to_string())])])
}

pub fn comma_cards(cards: &[Card]) -> Vec<N> {
    let mut output: Vec<N> = vec![];
    for c in cards {
        if !output.is_empty() {
            output.push(N::text(", "));
        }
        output.push(card(c));
    }
    output
}
