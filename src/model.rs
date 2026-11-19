//! Data and rules of the diary, independent of the terminal interface.
//! Diary owns a linked list of days; each day owns an ArrayList of entry texts.
//! No ready-made collection stores the days or entries.

use std::fmt;
use std::io::{self, Read, Write};

// One owned heap array. len counts entries; items.len() is the capacity.
// Slots 0..len contain Some(text); the remaining slots contain None.
pub struct ArrayList {
    items: Box<[Option<String>]>,
    pub len: usize,
}

impl ArrayList {
    pub fn new() -> Self {
        Self {
            items: Self::empty_array(8),
            len: 0,
        }
    }

    // Allocate empty slots only. Growth, insertion and deletion are our code.
    fn empty_array(capacity: usize) -> Box<[Option<String>]> {
        std::iter::repeat_with(|| None).take(capacity).collect()
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.items.get(index)?.as_deref()
    }

    pub fn push(&mut self, text: String) {
        if self.len == self.items.len() {
            // Full: allocate twice the space and move entries to the new array.
            let mut larger = Self::empty_array(self.items.len() * 2);
            for (i, slot) in larger.iter_mut().enumerate().take(self.len) {
                *slot = self.items[i].take();
            }
            // Replacing items frees the old, now empty array automatically.
            self.items = larger;
        }
        self.items[self.len] = Some(text);
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }
        // Move entries left to close the gap: [A, B, C] -> [A, C, None].
        for i in index..self.len - 1 {
            self.items[i] = self.items[i + 1].take();
        }
        self.len -= 1;
        self.items[self.len] = None;
        true
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
        let error = || "Enter a valid date in dd.MM.yyyy format.".to_owned();
        // "01.02.2026" -> "01" and "02.2026" -> "02" and "2026".
        let (day, rest) = s.split_once('.').ok_or_else(error)?;
        let (month, year) = rest.split_once('.').ok_or_else(error)?;
        // Each part must have exactly the given number of digits.
        let digits = |part: &str, width: usize| {
            part.len() == width && part.bytes().all(|c| c.is_ascii_digit())
        };
        if !digits(day, 2) || !digits(month, 2) || !digits(year, 4) {
            return Err(error());
        }
        // Only digits remain, so parsing cannot fail.
        let day: u8 = day.parse().unwrap();
        let month: u8 = month.parse().unwrap();
        let year: u16 = year.parse().unwrap();
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
    // Separate from add so the interface can reject a date before the text.
    pub fn check_date(&self, date: Date) -> Result<(), String> {
        if let Some(day) = self.day() {
            if date < day.date || day.next.as_ref().is_some_and(|next| date >= next.date) {
                return Err(
                    "Date must match the current day or be between it and the next day.".into(),
                );
            }
        }
        Ok(())
    }

    pub fn add(&mut self, date: Date, text: String) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("Entry text cannot be blank.".into());
        }
        self.check_date(date)?;
        if let Some(day) = self.link(self.current).as_mut() {
            if date == day.date {
                day.entries.push(text);
                return Ok(());
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

    // File format, one item per line:
    //   DENIK1
    //   01.01.2026 2      <- date and number of entries
    //   1                 <- number of lines of the first entry
    //   Hello
    //   2                 <- the second entry has two lines
    //   first line
    //   second line
    // The line count lets an entry contain newlines.
    pub fn save(&self, mut out: impl Write) -> io::Result<()> {
        writeln!(out, "DENIK1")?;
        let mut node = self.head.as_deref();
        while let Some(day) = node {
            writeln!(out, "{} {}", day.date, day.entries.len)?;
            for i in 0..day.entries.len {
                let text = day.entries.get(i).unwrap();
                writeln!(out, "{}", text.split('\n').count())?;
                writeln!(out, "{text}")?;
            }
            node = day.next.as_deref();
        }
        Ok(())
    }

    // Build a separate diary; the caller replaces its data only on success.
    pub fn load(mut input: impl Read) -> Result<Self, String> {
        let mut data = String::new();
        input.read_to_string(&mut data).map_err(|e| e.to_string())?;
        let mut lines = data.lines();
        if lines.next() != Some("DENIK1") {
            return Err("Invalid diary file.".into());
        }
        let mut diary = Self::new();
        while let Some(header) = lines.next() {
            let (date, count) = header.split_once(' ').ok_or("Invalid day header.")?;
            let date = Date::parse(date)?;
            let count: usize = count.parse().map_err(|_| "Invalid entry count.")?;
            if count == 0 || diary.day().is_some_and(|d| date <= d.date) {
                return Err("Days must be nonempty and sorted by date.".into());
            }
            for _ in 0..count {
                let line_count: usize = lines
                    .next()
                    .ok_or("Incomplete file.")?
                    .parse()
                    .map_err(|_| "Invalid line count.")?;
                // Join the entry's lines back together with newlines.
                let mut text = String::new();
                for i in 0..line_count {
                    if i > 0 {
                        text.push('\n');
                    }
                    text.push_str(lines.next().ok_or("Incomplete entry.")?);
                }
                // Reuse insertion validation, including rejecting blank text.
                diary.add(date, text)?;
            }
        }
        diary.current = 0;
        Ok(diary)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
