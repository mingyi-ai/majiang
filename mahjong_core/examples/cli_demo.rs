use std::io::{self, BufRead, Write};

use mahjong_core::board::{Board, BoardOutput, Decision, GameResult, Player};
use mahjong_core::round::{Event, State};
use mahjong_core::structs::Wind;

// ── Main ──

fn main() {
    loop {
        match main_menu() {
            MenuChoice::Quit => {
                println!("Goodbye!");
                break;
            }
            MenuChoice::StartGame => run_game(),
        }
    }
}

// ── Main Menu ──

enum MenuChoice {
    StartGame,
    Quit,
}

fn main_menu() -> MenuChoice {
    println!("\n━━━ Mahjong ━━━");
    println!("  1. Start new game");
    println!("  2. Quit");
    loop {
        print!("Choice: ");
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line).ok();
        match line.trim() {
            "1" => return MenuChoice::StartGame,
            "2" | "q" => return MenuChoice::Quit,
            _ => println!("  Invalid — enter 1 or 2."),
        }
    }
}

// ── Game Runner ──

fn run_game() {
    let mut state = State::new(Wind::East, Wind::East);
    state.init();

    let players = [CliPlayer, CliPlayer, CliPlayer, CliPlayer];
    let mut board = Board::new(state, players);

    println!("\n── Game Start ──");

    match board.run(print_event, print_on_end) {
        Ok(BoardOutput::GameConcluded(_)) => {
            println!("── Game Over ──");
        }
        Ok(BoardOutput::UserExited) => {
            println!("\nReturning to main menu.\n");
        }
        Err(e) => {
            println!("\nError: {:?}", e);
        }
    }
}

// ── CLI Player ──

struct CliPlayer;

impl Player for CliPlayer {
    fn decide(&self, options: &[Event]) -> Decision {
        println!();
        pick_event(options, "option")
    }
}

fn pick_event(options: &[Event], noun: &str) -> Decision {
    for (i, ev) in options.iter().enumerate() {
        println!("  {}. {:?}", i + 1, ev);
    }
    loop {
        print!("Choose {noun} (1-{}, q=quit to menu): ", options.len());
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line).ok();
        let line = line.trim();
        if line == "q" || line == "quit" {
            return Decision::Exit;
        }
        if let Ok(n) = line.parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return Decision::Pick(options[n - 1]);
            }
        }
        println!("  Invalid — enter 1-{} or q.", options.len());
    }
}

// ── Formatting helpers ──

fn print_event(e: &Event) {
    println!("  {:?}", e);
}

fn print_on_end(result: GameResult) {
    match result {
        GameResult::Hu { winner } => {
            println!("{} wins with Hu!", wind_name(winner))
        }
        GameResult::Draw => println!("Wall is empty — draw game."),
    }
}

fn wind_name(w: Wind) -> &'static str {
    match w {
        Wind::East => "East",
        Wind::South => "South",
        Wind::West => "West",
        Wind::North => "North",
    }
}
