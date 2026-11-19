# Terminal diary

School project described in [ZADANI.md](ZADANI.md). Rust 2024 edition,
Rust 1.85 or newer. Uses only the standard library: no external dependencies.
Interface and error messages are in English.

## Run

```sh
cargo run
```

The diary starts empty. Enter a command shown below the current day:

| Command | Action |
| --- | --- |
| `first`, `prev`, `next`, `last` | Select a day |
| `add` | Enter a date and multiline text |
| `delete` | Select an entry by number, preview it, confirm with `yes` |
| `save` | Save to `denik.txt` in the working directory |
| `load` | Replace the diary with `denik.txt`, selecting the first day |
| `quit` | Exit (save first; saving is manual) |

Dates use `dd.MM.yyyy`. Invalid dates are requested again; an empty line
cancels adding. The same date adds an entry to the current day. A later date
creates a day after the current one, provided it precedes the next day. Other
dates are rejected immediately, before the text is requested.

Finish entry text with a line containing only `.`. Use `..` to enter a literal
dot on its own line. Blank lines and spaces inside the entry are preserved.
Blank entries are rejected. Closing standard input exits without saving.

The current day is displayed with numbered entries and counts.
Deleting its last entry removes the day and selects the next day, or the
previous day when there is no next day. Cancelling deletion changes nothing.
A failed load leaves the current diary untouched.

## How the code works

- `src/main.rs`: terminal prompts, display, command loop, save/load handling.
- `src/model.rs`: date validation, custom lists, diary rules, file format.
- `src/tests.rs`: tests kept separate from the code you present.

Days form a custom singly linked list: `Option<Box<Day>>`. The selected day is
a zero-based index; finding it walks from the head, taking O(number of days).

Each day owns a custom `ArrayList`: one heap array of `Option<String>` slots.
It starts with capacity 8. When full, it allocates twice the capacity (16, 32,
64, ...), moves the strings into it, and releases the old array. `len` counts
entries; the array length is its capacity. Indexing is O(1), deletion shifts
later entries left in O(n), and appending is O(1) amortized (O(n) when growing).
Deletion keeps the allocated capacity for reuse.

`Box<[Option<String>]>` owns the array. The iterator in `empty_array` only
creates empty storage; resizing, moving entries, and list operations are
implemented explicitly. Strings are moved, not cloned. Memory is freed
automatically when the diary is dropped.

No `Vec`, built-in linked list, or map stores diary data. Comments explain
ownership, index calculations, node insertion/removal, and file parsing.
The file starts with `DENIK1`. Each day is a line with its date and entry
count; each entry is its number of lines followed by those lines, so multiline
text is preserved.

## Checks

```sh
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

Model tests cover dates, capacity doubling, shifting, insertion/deletion rules,
multiline UTF-8 save/load, and malformed files.

## Presentation order

1. Start with `run()` in main.rs: show the day, read a command, perform an action.
2. Explain `add()` and `delete()`: input validation and confirmation.
3. In model.rs, show `ArrayList`: one array, entry count, and capacity doubling.
4. Walk through `get`, `push`, and `remove`, using three entries as an example.
5. Show `Day` and `Diary`: a sorted linked list with a selected day index.
6. Finish with `save` and `load`: line counts preserve multiline text.
