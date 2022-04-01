use indicatif::{ProgressBar, ProgressStyle};
use ndarray::Array2;
use ndarray_npy::{NpzReader, NpzWriter};
use ordered_float::NotNan;
use priority_queue::DoublePriorityQueue;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::freq_to_entropy;

use super::handle_pinyin::HandlePinyin;
use super::lines_from_file;

pub struct Bot {
    pub all_idioms: Vec<String>,
    answer_candidates_index: Vec<usize>,
    // guess -> answer -> feedback (compressed)
    feedback_map: Array2<u32>,
    // feedback_map: Array<u32, Ix2>,
    pub round: u32,
    pub pinyin: HandlePinyin,
}

impl Bot {
    pub fn new() -> Bot {
        let all_idioms = lines_from_file("data/all_idioms.txt");
        let answer_candidates_index = (0..all_idioms.len()).collect();
        // let feedback_map = vec![vec![0; all_idioms.len()]; all_idioms.len()];
        let feedback_map = Array2::zeros((all_idioms.len(), all_idioms.len()));
        let pinyin = HandlePinyin::new("data/polyphones.json");
        let mut b = Bot {
            all_idioms,
            answer_candidates_index,
            feedback_map,
            round: 0,
            pinyin,
        };
        b.build_feedback_map();
        b
    }

    pub fn build_feedback_map(&mut self) {
        let feedback_map_path = "data/feedback_map.npz";
        if Path::new(feedback_map_path).exists() {
            let mut npz =
                NpzReader::new(File::open(feedback_map_path).expect("Cannot open feedback file"))
                    .expect("Cannot read feedback file");
            self.feedback_map = npz.by_name("").expect("Cannot read array");
        } else {
            let bar = ProgressBar::new(self.all_idioms.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar().template(
                    "{bar} {pos} / {len}, Elapsed: {elapsed_precise}, ETA: {eta_precise}",
                ),
            );
            for i in 0..self.all_idioms.len() {
                // for i in self.answer_candidates_index.iter() {
                for j in self.answer_candidates_index.iter() {
                    // let i = *i;
                    let j = *j;
                    self.feedback_map[[i, j]] = self
                        .pinyin
                        .diff(self.all_idioms[i].clone(), self.all_idioms[j].clone());
                }
                bar.inc(1);
            }
            bar.finish();
            let mut npz = NpzWriter::new(
                File::create(feedback_map_path).expect("Cannot create feedback file"),
            );
            npz.add_array("", &self.feedback_map)
                .expect("Cannot add array");
            npz.finish().expect("Cannot finish writing feedback file");
        }
    }

    pub fn motd(&self) {
        println!("欢迎使用汉兜 bot! 每轮请输入");
        println!("1. q:           退出");
        println!("2. r:           重新开始");
        println!("3. <字>:        汉兜的提示汉字");
        println!("4. <猜的词>");
        println!("   <汉兜的反馈> (16位数字, 分为四组, 分别表示每个字辅音、元音、调号、文字的正确性)");
        println!("     0 / <空格>: 无");
        println!("     1:          黄");
        println!("     2:          绿");
    }

    pub fn restart(&mut self) {
        self.answer_candidates_index = (0..self.all_idioms.len()).collect();
        self.round = 0;
    }

    // 让 bot 给出当前较优解
    pub fn bot_candidates(&self) -> Vec<(String, f64)> {
        let mut freq: HashMap<u32, u32> = HashMap::new();
        // let mut heap: BinaryHeap<(NotNan<f64>, String)> = BinaryHeap::new();
        let mut pq = DoublePriorityQueue::new();

        for i in 0..self.all_idioms.len() {
            // calc freq
            freq.clear();

            let mut is_possible_answer = false;
            for j in self.answer_candidates_index.iter() {
                let j = *j;
                let x = &self.feedback_map[[i, j]];
                *freq.entry(*x).or_insert(0) += 1;
                if i == j {
                    is_possible_answer = true;
                }
            }

            // calc entropy
            let mut e = freq_to_entropy(&freq, self.answer_candidates_index.len() as u32);

            if is_possible_answer {
                // magic number!
                e += 1.0 + 1e-5;
            }

            // enqueue
            if pq.len() >= 10 {
                pq.pop_min();
            }
            pq.push(self.all_idioms[i].clone(), NotNan::new(e).unwrap());
        }
        pq.into_sorted_iter()
            .map(|(s, e)| (s, f64::from(e)))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    // 猜单词, 并给出反馈, 供 bot 缩小解空间
    pub fn user_guess(&mut self, guess: String, feedback_string: String) {
        self.round += 1;
        let feedback_string = feedback_string.replace(" ", "0");
        let mut feedback_string_length = 0;
        let mut feedback = 0;
        for ch in feedback_string.chars() {
            feedback = feedback * 3 + ch.to_digit(10).unwrap();
            feedback_string_length += 1;
        }
        if feedback_string_length < 16 {
            feedback *= u32::pow(3, 16 - feedback_string_length);
        }
        if let Some(i) = self.all_idioms.iter().position(|x| x.eq(&guess)) {
            self.answer_candidates_index
                .drain_filter(|&mut j| self.feedback_map[[i, j]] != feedback);
        } else {
            self.answer_candidates_index.drain_filter(|&mut j| {
                self.pinyin.diff(guess.clone(), self.all_idioms[j].clone()) != feedback
            });
        }
    }

    // 提示字
    pub fn user_hint(&mut self, hint: String) {
        self.answer_candidates_index
            .drain_filter(|&mut i| !self.all_idioms[i].contains(&hint));
        // self.build_feedback_map();
    }

    pub fn print_status(&self) {
        println!(
            "Round {}: {} answers remain!",
            self.round,
            self.answer_candidates_index.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_bot() {
        let mut bot = Bot::new();
        bot.user_hint("不".to_string());
        bot.user_guess("雄鸡断尾".to_string(), "0010111100200010".to_string());
        let answer_candidates: Vec<String> = bot
            .answer_candidates_index
            .iter()
            .map(|i| bot.all_idioms[*i].clone())
            .collect();

        assert_eq!(answer_candidates, vec!["鸡犬不宁", "鸡犬不留"]);

        assert_eq!(
            bot.bot_candidates()
                .into_iter()
                .take(2)
                .map(|(s, _)| s)
                .collect::<Vec<_>>(),
            vec!["鸡犬不宁", "鸡犬不留"]
        );
    }
}
