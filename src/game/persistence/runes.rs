use unicode_segmentation::UnicodeSegmentation;

pub const FUTHARK: &'static str = include_str!("../../../assets/alphabet.txt");
pub const NON_FURTHARK: &'static str = "abcdefghijklmnopqrstuvwxyz12345678.";

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

pub fn retrieve_from_runes<T: crate::prelude::DeserializeOwned>() -> Result<T, String> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.get_text())
        .map_err(|e| e.to_string())
        .map(|text| {
            let futhark = parse_runes(&text, true);
            if futhark.len() > 0 {
                futhark
            } else {
                parse_runes(&text, false)
            }
        })
        .and_then(|bytes| postcard::from_bytes(&bytes).map_err(|e| e.to_string()))
}

pub fn store_in_runes<T: crate::prelude::Serialize>(t: T, futhark: bool) -> Option<String> {
    let runes = create_runes(t, futhark);
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(runes.clone()))
        .ok()
        .map(|_| runes)
}

pub fn create_runes<T: crate::prelude::Serialize>(t: T, futhark: bool) -> String {
    let data = postcard::to_allocvec(&t).unwrap();
    generate_runes(data.as_slice(), futhark)
}

#[cfg(test)]
mod runes_tests {
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
