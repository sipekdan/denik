//! Terminal interface. The custom lists and diary rules live in model.rs.
mod model;

use model::{Date, Diary};
use std::fs::File;
use std::io::{self, Write};

// Flush the prompt before reading. EOF exits rather than looping forever.
fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_owned())
}

fn show(diary: &Diary) {
    let count = diary.day().map_or(0, |day| day.entries.len);
    println!("\nDays: {} | Entries in current day: {count}", diary.len);
    if let Some(day) = diary.day() {
        println!("Date: {}", day.date);
        for i in 0..day.entries.len {
            println!("\n{}. {}", i + 1, day.entries.get(i).unwrap());
        }
    } else {
        println!("Diary is empty.");
    }
    println!("\nfirst | prev | next | last | add | delete | save | load | quit");
}

fn add(diary: &mut Diary) -> io::Result<()> {
    // Retry invalid formats, as required by the assignment. An empty line cancels.
    let date = loop {
        let input = prompt("Date (dd.MM.yyyy, empty to cancel): ")?;
        if input.trim().is_empty() {
            println!("Adding cancelled.");
            return Ok(());
        }
        match Date::parse(input.trim()) {
            Ok(date) => break date,
            Err(error) => println!("{error}"),
        }
    };
    // Reject a date in the wrong place now, so no text is typed in vain.
    if let Err(error) = diary.check_date(date) {
        println!("{error}");
        return Ok(());
    }
    println!("Enter text. Finish with a line containing only . (use .. for a literal .).");
    let mut text = String::new();
    loop {
        let line = prompt("> ")?;
        if line == "." {
            break;
        }
        // Blank lines and spaces are kept; only the terminator is special.
        text.push_str(if line == ".." { "." } else { &line });
        text.push('\n');
    }
    // Remove the last input separator, retaining intentional blank lines.
    text.pop();
    match diary.add(date, text) {
        Ok(()) => println!("Entry added."),
        Err(error) => println!("{error}"),
    }
    Ok(())
}

fn delete(diary: &mut Diary) -> io::Result<()> {
    let Some(day) = diary.day() else {
        println!("Diary is empty.");
        return Ok(());
    };
    // Displayed numbers start at 1; the ArrayList uses indices from 0.
    let number = match prompt("Entry number: ")?.parse::<usize>() {
        Ok(number) if number > 0 && number <= day.entries.len => number,
        _ => {
            println!("Invalid entry number.");
            return Ok(());
        }
    };
    let index = number - 1;
    let text = day.entries.get(index).unwrap();
    println!("\nDelete this entry?\n{text}");
    if prompt("Type yes to confirm: ")? == "yes" {
        diary.remove(index);
        println!("Entry deleted.");
    } else {
        println!("Deletion cancelled.");
    }
    Ok(())
}

fn save(diary: &Diary) -> io::Result<()> {
    diary.save(File::create("denik.txt")?)
}

fn run() -> io::Result<()> {
    let mut diary = Diary::new();
    println!("Diary - use save before quitting. File: denik.txt");
    loop {
        show(&diary);
        match prompt("Command: ")?.trim() {
            "first" => diary.current = 0,
            "prev" => diary.current = diary.current.saturating_sub(1),
            "next" => {
                if diary.current + 1 < diary.len {
                    diary.current += 1;
                }
            }
            "last" => diary.current = diary.len.saturating_sub(1),
            "add" => add(&mut diary)?,
            "delete" => delete(&mut diary)?,
            "save" => match save(&diary) {
                Ok(()) => println!("Saved to denik.txt."),
                Err(error) => println!("Save failed: {error}"),
            },
            "load" => {
                // A bad file cannot erase current data: replace only on success.
                match File::open("denik.txt")
                    .map_err(|e| e.to_string())
                    .and_then(Diary::load)
                {
                    Ok(loaded) => {
                        diary = loaded;
                        println!("Diary loaded.");
                    }
                    Err(error) => println!("Load failed: {error}"),
                }
            }
            "quit" => return Ok(()),
            _ => println!("Unknown command. Choose a command from the menu."),
        }
    }
}

fn main() {
    if let Err(error) = run() {
        if error.kind() != io::ErrorKind::UnexpectedEof {
            eprintln!("Input/output error: {error}");
            std::process::exit(1);
        }
    }
}
