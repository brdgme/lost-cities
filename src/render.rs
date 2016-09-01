use std::cmp;

use super::Game;
use card::{by_expedition, expeditions, Card};

use brdgme_color::GREY;
use brdgme_game::{Renderer, GameError};
use brdgme_markup::ast::{Node as N, Align as A, Row};

impl Renderer for Game {
    fn render(&self, player: Option<usize>) -> Result<Vec<N>, GameError> {
        let persp = player.unwrap_or(0);
        if persp > 1 {
            return Err(GameError::Internal("invalid player number".to_string()));
        }
        let mut layout: Vec<Row> = vec![vec![(A::Center,
                                              vec![
                N::Text("Round ".to_string()),
                N::Bold(vec![N::Text(format!("{}", self.round))]),
                N::Text(" of ".to_string()),
                N::Bold(vec![N::Text(format!("{}", super::ROUNDS))]),
            ])],
                                        vec![],
                                        vec![
                (A::Center, self.render_tableau(persp)),
            ]];
        if let Some(p) = player {
            // Not a spectator, also show hand.
            layout.append(&mut vec![vec![],
                                    vec![
                                        (A::Center, vec![
                                            N::Fg(GREY, vec![
                                                N::Text("Your hand".to_string())
                                            ]),
                                        ]),
                                        ],
                                    vec![
                    (A::Center, render_hand(&self.hands[p])),
                ]]);
        }
        Ok(vec![
            N::Table(layout),
        ])
    }
}

impl Game {
    fn render_tableau(&self, player: usize) -> Vec<N> {
        let p = cmp::min(player, 1);
        let mut rows: Vec<Row> = vec![];

        // Top half
        let mut top = render_tableau_cards(&self.expeditions[super::opponent(p)],
                                           vec![N::Fg(GREY,
                                                      vec![
                                                          N::Text("Them".to_string()),
                                                      ])]);
        top.reverse();
        rows.append(&mut top);

        // Blank row
        rows.push(vec![]);

        // Discards
        let mut discards: Row = vec![(A::Right,
                                      vec![N::Fg(GREY,
                                                 vec![
                                      N::Text("Discard".to_string()),
                                  ])])];
        for e in expeditions() {
            // Column spacing
            discards.push((A::Left,
                           vec![
                    N::Text("  ".to_string()),
                ]));

            discards.push((A::Center,
                           vec![if let Some(c) = self.available_discard(e) {
                                    card(&c)
                                } else {
                                    N::Fg(e.color(),
                                          vec![
                                             N::Text("--".to_string()),
                                         ])
                                }]));
        }
        rows.push(discards);

        // Blank row
        rows.push(vec![]);

        // Bottom half
        rows.append(&mut render_tableau_cards(&self.expeditions[p],
                                              vec![N::Fg(GREY,
                                                         vec![
                N::Text("You".to_string()),
            ])]));
        vec![
            N::Table(rows),
        ]
    }
}

fn render_tableau_cards(cards: &Vec<Card>, header: Vec<N>) -> Vec<Row> {
    let mut rows: Vec<Row> = vec![];
    let by_exp = by_expedition(cards);
    let mut largest: usize = 1;
    for e in expeditions() {
        largest = cmp::max(largest, by_exp.get(&e).unwrap_or(&vec![]).len());
    }
    for row_i in 0..largest {
        let mut row: Row = vec![if row_i == 0 {
                                    (A::Right, header.to_owned())
                                } else {
                                    (A::Left, vec![])
                                }];
        for e in expeditions() {
            // Column spacing
            row.push((A::Left, vec![]));
            match by_exp.get(&e).unwrap_or(&vec![]).get(row_i) {
                Some(c) => {
                    row.push((A::Center,
                              vec![
                                  card(c),
                              ]))
                }
                None => row.push((A::Left, vec![])),
            }
        }
        rows.push(row);
    }
    rows
}

fn render_hand(cards: &Vec<Card>) -> Vec<N> {
    let mut output: Vec<N> = vec![];
    let mut sorted = cards.to_owned();
    sorted.sort();
    for c in sorted {
        if output.len() > 0 {
            output.push(N::Text(" ".to_string()));
        }
        output.push(card(&c));
    }
    output
}

pub fn card(c: &Card) -> N {
    let &(e, _) = c;
    N::Bold(vec![N::Fg(e.color(),
                       vec![
                N::Text(card_text(c)),
            ])])
}

pub fn comma_cards(cards: &Vec<Card>) -> Vec<N> {
    let mut output: Vec<N> = vec![];
    for c in cards {
        if output.len() > 0 {
            output.push(N::Text(", ".to_string()));
        }
        output.push(card(c));
    }
    output
}

pub fn card_text(&(e, v): &Card) -> String {
    format!("{}{}", e, v)
}
