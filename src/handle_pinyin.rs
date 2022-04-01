use lazy_static::lazy_static;
use pinyin::ToPinyin;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct HandlePinyin {
    polyphones_dict: HashMap<String, String>,
}

impl HandlePinyin {
    pub fn new(polyphones_path: impl AsRef<Path>) -> HandlePinyin {
        let s = fs::read_to_string(polyphones_path).expect("Polyphones file not exists!");
        let d: HashMap<String, String> = serde_json::from_str(s.as_str()).unwrap();
        HandlePinyin { polyphones_dict: d }
    }

    pub fn diff(&self, phrase_s: String, phrase_t: String) -> u32 {
        let parsed_s = self.parse(phrase_s);
        let parsed_t = self.parse(phrase_t);

        let mut hint = vec![0u32; 16];
        let mut used = vec![false; 16];

        // println!("{:?}", parsed_s);
        // println!("{:?}", parsed_t);
        for i in 0..4 {
            for k in 0..4 {
                if parsed_s[i][k] == "" {
                    continue;
                }
                if parsed_s[i][k] == parsed_t[i][k] {
                    hint[i * 4 + k] = 2;
                    used[i * 4 + k] = true;
                } else {
                    for j in 0..4 {
                        if i == j {
                            continue;
                        }
                        if parsed_s[i][k] == parsed_t[j][k] && !used[i * 4 + k] {
                            hint[i * 4 + k] = 1;
                            used[i * 4 + k] = true;
                        }
                    }
                }
            }
        }

        // println!("{:?}", hint);

        let mut x = 0;
        for h in hint {
            x = x * 3 + h;
        }
        x
    }

    pub fn get_pinyin(&self, phrase: String) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new("(y|j|q|x)u([a-z]*[0-9]?)( |$)").unwrap();
        }
        let pinyin = if self.polyphones_dict.contains_key(&phrase) {
            self.polyphones_dict[&phrase].clone()
        } else {
            phrase
                .as_str()
                .to_pinyin()
                .into_iter()
                .map(|p| p.unwrap().with_tone_num_end().to_string().replace("ü", "v"))
                .collect::<Vec<String>>()
                .join(" ")
        };
        RE.replace_all(pinyin.as_str(), "${1}v${2}${3}").to_string()
    }

    pub fn parse(&self, phrase: String) -> Vec<Vec<String>> {
        lazy_static! {
            static ref RE: Regex = Regex::new("(b|p|m|f|d|t|n|l|g|k|h|j|q|r|x|w|y|zh|ch|sh|z|c|s)?(a|ai|an|ang|ao|e|ei|en|eng|er|i|ia|ian|iang|iao|ie|in|ing|io|iong|iu|o|ong|ou|u|ua|uai|uan|uang|ui|un|uo|v|van|ve|vn|ue)(\\d)?(?: |$)").unwrap();
        }
        let pinyin = self.get_pinyin(phrase.clone());
        let mut result = RE
            .captures_iter(pinyin.as_str())
            // word pinyin
            .map(|cap| {
                cap.iter()
                    .skip(1)
                    .map(|c| match c {
                        Some(s) => s.as_str().to_string(),
                        _ => "".to_string(),
                    })
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<_>>>();
        for (vec, ch) in result.iter_mut().zip(phrase.chars()) {
            vec.push(ch.to_string());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_pinyin() {
        let pinyin = HandlePinyin::new("data/polyphones.json");
        assert_eq!(
            pinyin.get_pinyin("礼崩乐坏".to_string()),
            "li3 beng1 yve4 huai4"
        );
        assert_eq!(
            pinyin.get_pinyin("颜丹鬓绿".to_string()),
            "yan2 dan1 bin4 lv4"
        );
        assert_eq!(
            pinyin.parse("礼崩乐坏".to_string()),
            vec![
                vec!["l", "i", "3", "礼"],
                vec!["b", "eng", "1", "崩"],
                vec!["y", "ve", "4", "乐"],
                vec!["h", "uai", "4", "坏"],
            ]
        );

        // println!("{:?}", pinyin.parse("不知所云".to_string()));
        // println!("{:?}", pinyin.parse("鸡犬不宁".to_string()));
        assert_eq!(
            pinyin.diff(String::from("不知所云"), String::from("鸡犬不宁")),
            21336621
        );
        assert_eq!(
            pinyin.diff(String::from("礼崩乐坏"), String::from("快快乐乐")),
            // 14349570
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 1, 2, 0]
                .iter()
                .fold(0, |acc, x| acc * 3 + x)
        );
    }
}
