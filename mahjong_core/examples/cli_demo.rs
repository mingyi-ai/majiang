use std::io::{self, BufRead, Write};

use mahjong_core::board::{Board, BoardError, Player, StepResult};
use mahjong_core::round::{Event, Output, State};
use mahjong_core::structs::{Tile, Wind};

fn main() {
    let mut state = State::new(Wind::East, Wind::East);
    state.init();

    let players = [CliPlayer, CliPlayer, CliPlayer, CliPlayer];
    let mut board = Board::new(state, players);

    println!("━━━ Mahjong CLI Demo ━━━");
    println!("Type the number of your choice, or 'q' to quit.\n");

    loop {
        match board.step() {
            Ok(StepResult::Waiting { output }) => {
                let input = board.prompt(&output);
                match board.decide(&output, input) {
                    Ok(event) => {
                        if matches!(event, Event::Hu { .. }) {
                            println!("\n🎉 {} wins with Hu!\n", wind_name(event.seat()));
                            break;
                        }
                    }
                    Err(BoardError::InvalidInput(msg)) => {
                        println!("[error] {msg}");
                    }
                    Err(BoardError::WallEmpty) => unreachable!(),
                }
            }
            Ok(StepResult::Over) => {
                println!("\nWall is empty — draw game.");
                break;
            }
            Err(e) => {
                println!("[error] {e:?}");
                break;
            }
        }
    }

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
                println!(
                    "[{}] Reaction: pick an option",
                    wind_name(seat)
                );
                pick_event(options, "option")
            }
            Output::NeedDrawTile => unreachable!(),
        }
    }
}

fn pick_event(options: &[Event], noun: &str) -> Event {
    for (i, ev) in options.iter().enumerate() {
        println!("  {}. {}", i + 1, describe_event(ev));
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

fn wind_name(w: Wind) -> &'static str {
    match w {
        Wind::East => "East",
        Wind::South => "South",
        Wind::West => "West",
        Wind::North => "North",
    }
}

fn tile_name(t: Tile) -> String {
    match t.get_type() {
        mahjong_core::structs::TileType::Character => {
            let n = ((t as u8 & 0x3F) >> 2) + 1;
            format!("{n} Wan")
        }
        mahjong_core::structs::TileType::Dot => {
            let n = ((t as u8 & 0x3F) >> 2) + 1;
            format!("{n} Tong")
        }
        mahjong_core::structs::TileType::Bamboo => {
            let n = ((t as u8 & 0x3F) >> 2) + 1;
            format!("{n} Tiao")
        }
        mahjong_core::structs::TileType::Wind => match t {
            Tile::East => "East Wind",
            Tile::South => "South Wind",
            Tile::West => "West Wind",
            Tile::North => "North Wind",
            _ => unreachable!(),
        }
        .into(),
        mahjong_core::structs::TileType::Dragon => match t {
            Tile::Red => "Red Dragon",
            Tile::Green => "Green Dragon",
            Tile::White => "White Dragon",
            _ => unreachable!(),
        }
        .into(),
        mahjong_core::structs::TileType::Flower => format!("{:?}", t),
    }
}

fn describe_event(ev: &Event) -> String {
    match ev {
        Event::Skip { .. } => "Skip".into(),
        Event::Discard { tile, .. } => format!("Discard {}", tile_name(*tile)),
        Event::Chow { start, .. } => {
            format!(
                "Chow {}-{}-{} (start: {})",
                tile_name(*start),
                tile_name(start.next()),
                tile_name(start.next().next()),
                tile_name(*start),
            )
        }
        Event::Pong { tile, .. } => format!("Pong {}", tile_name(*tile)),
        Event::Kong { tile, .. } => format!("Kong {}", tile_name(*tile)),
        Event::Hu { .. } => "Hu!".into(),
        Event::ConcealedKong { tile, .. } => {
            format!("Concealed Kong {}", tile_name(*tile))
        }
        Event::AddedKong { tile, .. } => {
            format!("Added Kong {}", tile_name(*tile))
        }
        _ => format!("{:?}", ev),
    }
}
