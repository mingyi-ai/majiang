use std::io::{self, BufRead, Write};

use mahjong_core::board::{Board, GameResult, Player};
use mahjong_core::round::{Event, Output, State};
use mahjong_core::structs::Wind;

fn main() {
    let mut state = State::new(Wind::East, Wind::East);
    state.init();

    let players = [CliPlayer, CliPlayer, CliPlayer, CliPlayer];
    let mut board = Board::new(state, players);

    println!("━━━ Mahjong CLI Demo ━━━");
    println!("Type the number of your choice, or 'q' to quit.\n");

    board
        .run(print_event, |result| match result {
            GameResult::Hu { winner } => {
                println!("\n🎉 {} wins with Hu!\n", wind_name(winner));
            }
            GameResult::Draw => {
                println!("\nWall is empty — draw game.");
            }
        })
        .unwrap();

    println!("── Game Over ──");
}

// ── CLI Player ──

struct CliPlayer;

impl Player for CliPlayer {
    fn decide(&self, output: &Output) -> Event {
        println!();
        match output {
            Output::NeedSelfAction { options } => {
                let seat = options.first().map(|e| e.seat()).unwrap();
                println!("[{}] Self-action: pick one", wind_name(seat));
                pick_event(options, "action")
            }
            Output::NeedDiscard { options } => {
                let seat = options.first().map(|e| e.seat()).unwrap();
                println!("[{}] Discard: pick a tile", wind_name(seat));
                pick_event(options, "tile")
            }
            Output::NeedReactions { options } => {
                let seat = options.first().map(|e| e.seat()).unwrap();
                println!("[{}] Reaction: pick an option", wind_name(seat));
                pick_event(options, "option")
            }
            Output::NeedDrawTile => unreachable!(),
        }
    }
}

fn pick_event(options: &[Event], noun: &str) -> Event {
    for (i, ev) in options.iter().enumerate() {
        println!("  {}. {:?}", i + 1, ev);
    }
    loop {
        print!("Choose {noun} (1-{}, q to quit): ", options.len());
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line).ok();
        let line = line.trim();
        if line == "q" || line == "quit" {
            println!("Game ended by player.");
            std::process::exit(0);
        }
        if let Ok(n) = line.parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return options[n - 1];
            }
        }
        println!("  Invalid — enter 1-{} or q.", options.len());
    }
}

// ── Formatting helpers ──

/// Print an event to the event log.
fn print_event(e: &Event) {
    println!("  {:?}", e);
}

fn wind_name(w: Wind) -> &'static str {
    match w {
        Wind::East => "East",
        Wind::South => "South",
        Wind::West => "West",
        Wind::North => "North",
    }
}
