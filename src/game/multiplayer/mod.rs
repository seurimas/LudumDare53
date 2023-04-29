use unicode_segmentation::UnicodeSegmentation;

use crate::prelude::*;

pub const FUTHARK: &'static str = include_str!("../../../assets/alphabet.txt");
pub const NON_FURTHARK: &'static str = "abcdefghijklmnopqrstuvwxyz12345678.";

pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {}
}

pub fn parse_runes_to_points(runes: &str, futhark: bool) -> Vec<u8> {
    let alphabet = if futhark { FUTHARK } else { NON_FURTHARK };
    let mut results = Vec::new();
    for rune in runes.graphemes(true) {
        let mut alphabet = alphabet.graphemes(true);
        if let Some(idx) = alphabet.position(|alpha| alpha == rune) {
            results.push(idx as u8);
            if idx == 32 {
                break;
            }
        }
    }
    results
}

pub fn points_to_bytes(points: Vec<u8>) -> Vec<u8> {
    let mut results = Vec::new();
    let mut bits: u16 = 0;
    let mut offset = 0;
    for point in points {
        if offset == 0 {
            bits = point as u16;
            offset = 5;
        } else {
            bits |= (point as u16) << offset;
            offset += 5;
        }
        if offset >= 8 {
            results.push((bits & 0xff) as u8);
            bits >>= 8;
            offset -= 8;
        }
    }
    results
}

pub fn parse_runes(runes: &str, futhark: bool) -> Vec<u8> {
    let points = parse_runes_to_points(runes, futhark);
    points_to_bytes(points)
}

fn bytes_to_points(bytes: &[u8]) -> Vec<u8> {
    let mut results = Vec::new();
    let mut bits: u16 = 0;
    let mut offset = 0;
    for byte in bytes {
        bits |= (*byte as u16) << offset;
        offset += 8;
        while offset >= 5 {
            results.push((bits & 0x1f) as u8);
            bits >>= 5;
            offset -= 5;
        }
    }
    if offset != 0 {
        results.push(bits as u8);
    }
    results
}

pub fn generate_runes(bytes: &[u8], futhark: bool) -> String {
    let points = bytes_to_points(bytes);
    let alphabet = if futhark { FUTHARK } else { NON_FURTHARK };
    points
        .iter()
        .map(|point| alphabet.graphemes(true).nth(*point as usize).unwrap())
        .collect()
}

#[cfg(test)]
mod multiplayer_tests {
    use super::*;

    #[test]
    fn test_parse_runes_to_points() {
        assert_eq!(parse_runes_to_points("ᚠᛌ", true), vec![0, 32]);
        assert_eq!(parse_runes_to_points("ᚢᛌ", true), vec![1, 32]);
        assert_eq!(parse_runes_to_points("ab", false), vec![0, 1]);
        assert_eq!(parse_runes_to_points("cd", false), vec![2, 3]);
    }

    #[test]
    fn test_parse_runes() {
        assert_eq!(parse_runes("ᚠᚠ", true), vec![0b00000]);
        assert_eq!(parse_runes("ᚢᚠᛌ", true), vec![0b00001]);
        assert_eq!(parse_runes("ᚠᛁᚢᚠ", true), vec![64, 5]);
        assert_eq!(parse_runes("akba", false), vec![64, 5]);
        assert_eq!(
            parse_runes("akbaakbaakba", false),
            vec![64, 5, 0, 84, 0, 64, 5]
        );
    }

    #[test]
    fn test_generate_runes() {
        assert_eq!(generate_runes(&[0b00000000], true), "ᚠᚠ");
        assert_eq!(generate_runes(&[0b00000001], true), "ᚢᚠ");
        assert_eq!(generate_runes(&[0b100000], true), "ᚠᚢ");
        assert_eq!(generate_runes(&[64, 5], false), "akba");
        assert_eq!(
            generate_runes(&[64, 5, 0, 84, 0, 64, 5], true),
            "ᚠᛁᚢᚠᚠᛁᚢᚠᚠᛁᚢᚠ"
        );
    }
}
