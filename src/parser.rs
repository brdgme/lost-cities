use card;

named!(parse_card< &[u8], card::Card >,
    map_res!(tuple!(parse_suit, parse_value), make_card));
named!(parse_suit< &[u8], card::Expedition >, alt!(
    parse_suit_red
));
named!(parse_suit_red< &[u8], card::Expedition >, map_res!(alt!(
    char!('r') |
    char!('R')
), char_to_expedition));
named!(parse_value< card::Value >, map_res!(char!('x'), char_to_value));
named!(pub play_command< &[u8], (&[u8], card::Card) >, tuple!(
    tag!("play"),
    parse_card
));
named!(pub draw_command, tag!("draw"));

fn char_to_expedition(c: char) -> Result<card::Expedition, String> {
    match c {
        'r' | 'R' => Ok(card::Expedition::Red),
        _ => Err("unknown expedition".to_string()),
    }
}

fn char_to_value(c: char) -> Result<card::Value, String> {
    match c {
        'x' | 'X' => Ok(card::Value::Investment),
        _ => Err("unknown value".to_string()),
    }
}

fn make_card(t: (card::Expedition, card::Value)) -> Result<card::Card, String> {
    let (e, v) = t;
    Ok(card::Card {
        expedition: e,
        value: v,
    })
}
