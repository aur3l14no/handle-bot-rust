#![feature(binary_heap_into_iter_sorted)]
#![feature(drain_filter)]

pub mod bot;
pub mod handle_pinyin;

use std::{
    collections::HashMap,
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
};

pub fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

pub fn freq_to_entropy(freq: &HashMap<u32, u32>, total: u32) -> f64 {
    let mut e = 0.0;
    for x in freq.values() {
        let p = *x as f64 / total as f64;
        e += p * (-f64::log2(p));
    }
    e
}

pub fn format_radix(mut x: u32, radix: u32) -> String {
    let mut result = vec![];

    loop {
        let m = x % radix;
        x = x / radix;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(m, radix).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_from_file_test() {
        let lines = lines_from_file("data/idioms.txt");
        assert_eq!(lines.len(), 26353);
    }

    #[test]
    fn test_entropy() {
        let freq_1 = HashMap::from([
            (1, 1),
            (2, 1),
        ]);
        let freq_2 = HashMap::from([
            (1, 1),
        ]);
        assert_eq!(freq_to_entropy(&freq_1, 2), 1.0);
        assert_eq!(freq_to_entropy(&freq_2, 1), 0.0);
    }
}
