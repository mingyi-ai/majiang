use std::io::{self, Write};

pub trait TerminalChoiceExt<T> {
    fn choose_with<F>(&self, prompt: &str, fmt: F) -> Option<T>
    where
        T: Copy,
        F: Fn(&T) -> String;

    fn choose_debug(&self, prompt: &str) -> Option<T>
    where
        T: Copy + std::fmt::Debug,
    {
        self.choose_with(prompt, |t| format!("{:?}", t))
    }
}

impl<T> TerminalChoiceExt<T> for [T] {
    fn choose_with<F>(&self, prompt: &str, fmt: F) -> Option<T>
    where
        T: Copy,
        F: Fn(&T) -> String,
    {
        if self.is_empty() {
            println!("No options available.");
            return None;
        }

        for (i, item) in self.iter().enumerate() {
            println!("{:2}: {}", i, fmt(item));
        }

        loop {
            print!("{prompt} ");
            io::stdout().flush().ok();

            let mut line = String::new();
            if io::stdin().read_line(&mut line).is_err() {
                println!("Input error, try again");
                continue;
            }

            match line.trim().parse::<usize>() {
                Ok(idx) if idx < self.len() => return Some(self[idx]),
                _ => {
                    println!("Invalid index, try again");
                    continue;
                }
            }
        }
    }
}
