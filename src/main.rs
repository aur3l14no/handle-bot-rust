use std::process::exit;

use handle_bot_rust::bot::Bot;

fn main() {
    let mut bot = Bot::new();
    bot.motd();

    let stdin = std::io::stdin();

    // bot loop
    loop {
        // ask for hint / feedback / (q)uit
        bot.print_status();
        let mut line = String::new();

        for (s, e) in bot.bot_candidates() {
            println!("{}: {}", s, e);
        }

        loop {
            stdin.read_line(&mut line).unwrap();
            line.pop();
            if line.chars().next().unwrap() == 'q' {
                println!("Bye~");
                exit(0);
            } else if line.chars().next().unwrap() == 'r' {
                println!("Restart");
                bot.restart();
                bot.print_status();
            } else {
                if line.chars().count() == 1 {
                    let hint = line.clone();
                    bot.user_hint(hint);
                } else {
                    let guess = line.clone();
                    line.clear();
                    stdin.read_line(&mut line).unwrap();
                    line.pop();
                    let feedback_string = line.clone();
                    bot.user_guess(guess, feedback_string);
                }
                bot.print_status();
                for (s, e) in bot.bot_candidates() {
                    println!("{}: {}", s, e);
                }
            }
            line.clear();
        }
    }
}
