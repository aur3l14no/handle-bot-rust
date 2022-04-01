use handle_bot_rust::{bot::Bot, format_radix};
use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    let mut bot = Bot::new();

    let bar = ProgressBar::new(bot.all_idioms.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{bar} {pos} / {len}, {msg}, Elapsed: {elapsed_precise}, ETA: {eta_precise}"),
    );

    let mut total_rounds = 0;
    let mut trials = 0;
    let all_idioms = bot.all_idioms.clone();
    for answer in all_idioms.iter() {
        trials += 1;
        bot.restart();
        let answer = answer.clone();
        loop {
            let (guess, _) = bot.bot_candidates().into_iter().next().unwrap();
            let feedback = bot.pinyin.diff(guess.clone(), answer.clone());
            let feedback_s = format_radix(feedback, 3);
            if feedback == 43046720 {
                total_rounds += bot.round + 1;
                break;
            }
            bot.user_guess(guess, feedback_s);
        }
        bar.inc(1);
        bar.set_message(format!("{}", total_rounds as f64 / trials as f64));
    }
    bar.finish();
    println!("{}", total_rounds as f64 / trials as f64);
}
