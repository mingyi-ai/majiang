use std::io::{self, BufRead, Write};

use mahjong_core::board::{Board, BoardOutput, Decision, GameResult, Player};
use mahjong_core::round::PlayerAction;
use mahjong_core::structs::Wind;
use rand::SeedableRng;
use rand::rngs::StdRng;

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
    // Seeded RNG for deterministic games (useful for debugging).
    let mut rng = StdRng::seed_from_u64(42);

    // Only the East seat prompts for input; other seats auto-play.
    let p = DemoPlayer { is_cli: true };
    let dummy = DemoPlayer { is_cli: false };
    let mut board = Board::new(
        Wind::East,
        Wind::East,
        [&p, &dummy, &dummy, &dummy],
        &mut rng,
    );

    println!("\n── Game Start ──\n");

    match board.run(
        |event| println!("  [deal] {:?}", event),
        |event| println!("  {:?}", event),
    ) {
        Ok(BoardOutput::GameConcluded(result)) => {
            match result {
                GameResult::Hu { winner } => {
                    println!("\n🎉 {} wins with Hu!\n", wind_name(winner));
                }
                GameResult::Draw => {
                    println!("\nWall is empty — draw game.");
                }
            }
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

// ── Demo Player ──

struct DemoPlayer {
    is_cli: bool,
}

impl Player for DemoPlayer {
    fn decide(&self, options: &[PlayerAction]) -> Decision {
        if self.is_cli {
            println!();
            for (i, action) in options.iter().enumerate() {
                println!("  {}. {:?}", i + 1, action);
            }
            loop {
                print!(
                    "Choose option (1-{}, q=quit to menu): ",
                    options.len()
                );
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
        } else {
            // Dummy: always pick the first option.
            Decision::Pick(options[0])
        }
    }
}

// ── Formatting helpers ──

fn wind_name(w: Wind) -> &'static str {
    match w {
        Wind::East => "East",
        Wind::South => "South",
        Wind::West => "West",
        Wind::North => "North",
    }
}
