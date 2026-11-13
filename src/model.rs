//! Data and rules of the diary, independent of the terminal interface.
//! Diary owns a linked list of days; each day owns an ArrayList of entry texts.
//! No ready-made collection stores the days or entries.

use std::fmt;
use std::io::{self, Read, Write};

// Fixed arrays linked together: [0..7] -> [8..15] -> ...
// Some(text) is occupied, None is empty. Box owns the next block.
const BLOCK: usize = 8;
#[derive(Default)]
struct Block {
    items: [Option<String>; BLOCK],
    next: Option<Box<Block>>,
}

#[derive(Default)]
// len is the number of entries; occupied slots have no gaps.
pub struct ArrayList {
    head: Block,
    pub len: usize,
}

impl ArrayList {
    pub fn new() -> Self {
        Self::default()
    }

    // Index 10: block 10 / 8 = 1, slot 10 % 8 = 2.
    pub fn get(&self, index: usize) -> Option<&str> {
        if index >= self.len {
            return None;
        }
        let mut block = &self.head;
        for _ in 0..index / BLOCK {
            block = block.next.as_deref()?;
        }
        block.items[index % BLOCK].as_deref()
    }

    // Find a writable slot, creating missing blocks when appending.
    fn slot(&mut self, index: usize) -> &mut Option<String> {
        let mut block = &mut self.head;
        for _ in 0..index / BLOCK {
            block = block.next.get_or_insert_default();
        }
        &mut block.items[index % BLOCK]
    }

    // Append at len, allocating another block only when needed.
    pub fn push(&mut self, text: String) {
        *self.slot(self.len) = Some(text);
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }
        // Move later entries left: [A, B, C] deleting B becomes [A, C].
        // take() moves a String without copying it, leaving None behind.
        for i in index..self.len - 1 {
            let next = self.slot(i + 1).take();
            *self.slot(i) = next;
        }
        self.len -= 1;
        // Also handles deleting the last entry, where the shift loop is empty.
        self.slot(self.len).take();
        true
    }
}

// Detach links one at a time to avoid recursive destruction of a long list.
impl Drop for ArrayList {
    fn drop(&mut self) {
        let mut next = self.head.next.take();
        while let Some(mut block) = next {
            next = block.next.take();
        }
    }
}

// Field order makes derived comparisons chronological: year, month, day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    pub fn parse(s: &str) -> Result<Self, String> {
        let b = s.as_bytes();
        let error = || "Enter a valid date in dd.MM.yyyy format.".to_owned();
        // Check ASCII first so the string slices below are safe.
        if b.len() != 10
            || b[2] != b'.'
            || b[5] != b'.'
            || !b
                .iter()
                .enumerate()
                .all(|(i, c)| i == 2 || i == 5 || c.is_ascii_digit())
        {
            return Err(error());
        }
        let day = s[..2].parse::<u8>().map_err(|_| error())?;
        let month = s[3..5].parse::<u8>().map_err(|_| error())?;
        let year = s[6..].parse::<u16>().map_err(|_| error())?;
        // Gregorian rule: divisible by 4, except centuries not divisible by 400.
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let max = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => 0,
        };
        if year == 0 || day == 0 || day > max {
            return Err(error());
        }
        Ok(Self { year, month, day })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}.{:02}.{:04}", self.day, self.month, self.year)
    }
}

// A node of the singly linked list. Entries belong to this day.
pub struct Day {
    pub date: Date,
    pub entries: ArrayList,
    next: Option<Box<Day>>,
}

#[derive(Default)]
// Sorted linked days. current is an index, not a pointer; 0 when empty.
pub struct Diary {
    head: Option<Box<Day>>,
    pub current: usize,
    pub len: usize,
}

impl Diary {
    pub fn new() -> Self {
        Self::default()
    }

    // Find the current day by walking from the head.
    pub fn day(&self) -> Option<&Day> {
        let mut node = self.head.as_deref()?;
        for _ in 0..self.current {
            node = node.next.as_deref()?;
        }
        Some(node)
    }

    // Return head or a previous node's next, so either can be reconnected.
    fn link(&mut self, index: usize) -> &mut Option<Box<Day>> {
        let mut link = &mut self.head;
        for _ in 0..index {
            link = &mut link.as_mut().expect("valid day index").next;
        }
        link
    }

    // Same date appends text. A new date must follow the current day and
    // precede its successor; with no successor, any later date is allowed.
    pub fn add(&mut self, date: Date, text: String) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("Entry text cannot be blank.".into());
        }
        if let Some(day) = self.link(self.current).as_mut() {
            if date == day.date {
                day.entries.push(text);
                return Ok(());
            }
            if date < day.date || day.next.as_ref().is_some_and(|next| date >= next.date) {
                return Err(
                    "Date must match the current day or be between it and the next day.".into(),
                );
            }
            self.current += 1;
        }
        let mut entries = ArrayList::new();
        entries.push(text);
        let link = self.link(self.current);
        // Insert new day before the old successor, keeping the rest of the list.
        *link = Some(Box::new(Day {
            date,
            entries,
            next: link.take(),
        }));
        self.len += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if self.len == 0 {
            return false;
        }
        let link = self.link(self.current);
        let day = link.as_mut().unwrap();
        if !day.entries.remove(index) {
            return false;
        }
        // Remove an empty day by making its owning link point to its successor.
        // The same index then selects the next day. At the end, move back one.
        if day.entries.len == 0 {
            *link = day.next.take();
            self.len -= 1;
            if self.current >= self.len {
                self.current = self.len.saturating_sub(1);
            }
        }
        true
    }

    // Format: magic header, then date/count and byte-length-prefixed texts.
    // Byte lengths allow text to contain newlines and UTF-8 characters.
    pub fn save(&self, mut out: impl Write) -> io::Result<()> {
        out.write_all(b"DENIK1\n")?;
        let mut node = self.head.as_deref();
        while let Some(day) = node {
            writeln!(out, "{} {}", day.date, day.entries.len)?;
            for i in 0..day.entries.len {
                let text = day.entries.get(i).unwrap();
                writeln!(out, "{}", text.len())?;
                out.write_all(text.as_bytes())?;
                out.write_all(b"\n")?;
            }
            node = day.next.as_deref();
        }
        Ok(())
    }

    // Build a separate diary; the caller replaces its data only on success.
    pub fn load(mut input: impl Read) -> Result<Self, String> {
        let mut data = String::new();
        input.read_to_string(&mut data).map_err(|e| e.to_string())?;
        let mut rest = data.strip_prefix("DENIK1\n").ok_or("Invalid diary file.")?;
        let mut diary = Self::new();
        // Read a metadata line and advance the remaining input.
        fn line<'a>(rest: &mut &'a str) -> Result<&'a str, String> {
            let (line, tail) = rest.split_once('\n').ok_or("Incomplete file.")?;
            *rest = tail;
            Ok(line)
        }
        while !rest.is_empty() {
            let header = line(&mut rest)?;
            let (date, count) = header.split_once(' ').ok_or("Invalid day header.")?;
            let date = Date::parse(date)?;
            let count: usize = count.parse().map_err(|_| "Invalid entry count.")?;
            if count == 0 || diary.day().is_some_and(|d| date <= d.date) {
                return Err("Days must be nonempty and sorted by date.".into());
            }
            for _ in 0..count {
                let size: usize = line(&mut rest)?
                    .parse()
                    .map_err(|_| "Invalid text length.")?;
                // get() also rejects lengths that split a UTF-8 character.
                let text = rest.get(..size).ok_or("Invalid text length.")?;
                let tail = rest
                    .get(size..)
                    .and_then(|s| s.strip_prefix('\n'))
                    .ok_or("Incomplete entry.")?;
                // Reuse insertion validation, including rejecting blank text.
                diary.add(date, text.to_owned())?;
                rest = tail;
            }
        }
        diary.current = 0;
        Ok(diary)
    }
}

// Detach each day to avoid recursive drops on a long diary.
impl Drop for Diary {
    fn drop(&mut self) {
        let mut next = self.head.take();
        while let Some(mut day) = next {
            next = day.next.take();
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
