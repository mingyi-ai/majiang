use std::io::{self, BufRead, Write};

use mahjong_core::{
    Engine, EngineOutput, GameEvent, GameResult, Player, PlayerAction,
    PlayerDecision, Wind,
};
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
    let mut rng = StdRng::seed_from_u64(42);

    let p = DemoPlayer { is_cli: true };
    let dummy = DemoPlayer { is_cli: false };
    let mut engine = Engine::new(
        Wind::East,
        Wind::East,
        [&p, &dummy, &dummy, &dummy],
        &mut rng,
    );

    println!("\n── Game Start ──\n");

    // Deal callback: show raw event for the CLI seat (East), hide tile
    // identity for other seats to mimic private-per-player dealing.
    match engine.run(
        |event| print_deal_event(event, Wind::East),
        |event| println!("  {:?}", event),
    ) {
        Ok(EngineOutput::GameConcluded(result)) => {
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
        Ok(EngineOutput::UserExited) => {
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
    fn decide(&self, options: &[PlayerAction]) -> PlayerDecision {
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
                    return PlayerDecision::Exit;
                }
                if let Ok(n) = line.parse::<usize>() {
                    if n >= 1 && n <= options.len() {
                        return PlayerDecision::Pick(options[n - 1]);
                    }
                }
                println!("  Invalid — enter 1-{} or q.", options.len());
            }
        } else {
            PlayerDecision::Pick(options[0])
        }
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

/// Show the full deal event for the CLI seat and for any flower tiles
/// (which are exposed to all players). For other seats' non-flower tiles,
/// only announce which seat received a tile.
fn print_deal_event(event: &GameEvent, reveal: Wind) {
    match event {
        GameEvent::DrawTile { seat, tile }
            if *seat == reveal || tile.is_flower() =>
        {
            println!("  [deal] {:?}", event);
        }
        GameEvent::DrawTile { seat, .. } => {
            println!("  [deal] Tile drawn for {:?}", seat);
        }
        _ => {}
    }
}
