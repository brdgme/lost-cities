use std::cmp;

use super::PlayerState;
use card::{by_expedition, expeditions, Card};

use brdgme_color::GREY;
use brdgme_game::Renderer;
use brdgme_markup::ast::{Node as N, Align as A, Row};

impl Renderer for PlayerState {
    fn render(&self) -> Vec<N> {
        let mut layout: Vec<Row> = vec![vec![(A::Center,
                                              vec![
                N::text("Round "),
                N::Bold(vec![N::text(format!("{}", self.round))]),
                N::text(" of "),
                N::Bold(vec![N::text(format!("{}", super::ROUNDS))]),
            ])],
                                        vec![],
                                        vec![
                (A::Center, self.render_tableau()),
            ]];
        if let Some(ref h) = self.hand {
            layout.append(&mut vec![vec![],
                                    vec![
                                        (A::Center, vec![
                                            N::Fg(GREY, vec![
                                                N::text("Your hand")
                                            ]),
                                        ]),
                                        ],
                                    vec![
                    (A::Center, render_hand(&h)),
                ]]);
        }
        vec![N::Table(layout)]
    }
}

impl PlayerState {
    fn render_tableau(&self) -> Vec<N> {
        let p = cmp::min(self.player.unwrap_or(0), 1);
        let mut rows: Vec<Row> = vec![];

        // Top half
        let mut top = render_tableau_cards(&self.expeditions[super::opponent(p)],
                                           &[N::Fg(GREY,
                                                   vec![
                                                          N::text("Them"),
                                                      ])]);
        top.reverse();
        rows.append(&mut top);

        // Blank row
        rows.push(vec![]);

        // Discards
        let mut discards: Row = vec![(A::Right,
                                      vec![N::Fg(GREY,
                                                 vec![
                                      N::text("Discard"),
                                  ])])];
        for e in expeditions() {
            // Column spacing
            discards.push((A::Left,
                           vec![
                    N::text("  "),
                ]));

            discards.push((A::Center,
                           vec![if let Some(v) = self.discards.get(&e) {
                                    card(&(e, *v))
                                } else {
                                    N::Fg(e.color(),
                                          vec![
                                             N::text("--"),
                                         ])
                                }]));
        }
        rows.push(discards);

        // Blank row
        rows.push(vec![]);

        // Bottom half
        rows.append(&mut render_tableau_cards(&self.expeditions[p],
                                              &[N::Fg(GREY,
                                                      vec![
                N::text("You"),
            ])]));
        vec![
            N::Table(rows),
        ]
    }
}

fn render_tableau_cards(cards: &[Card], header: &[N]) -> Vec<Row> {
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
    let e = c.0;
    N::Bold(vec![N::Fg(e.color(),
                       vec![
                N::text(card_text(c)),
            ])])
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

pub fn card_text(c: &Card) -> String {
    let &(e, v) = c;
    format!("{}{}", e, v)
}
