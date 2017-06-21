extern crate brdgme_cmd;
extern crate lost_cities;

use lost_cities::Game;
use brdgme_cmd::cli::cli;

use std::io;

fn main() {
    cli::<Game, _, _>(io::stdin(), &mut io::stdout());
}
