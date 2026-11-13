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

Dates use `dd.MM.yyyy`. Invalid dates are requested again. The same date adds
an entry to the current day. A later date creates a day after the current one,
provided it precedes the next day. Other dates are rejected.

Finish entry text with a line containing only `.`. Use `..` to enter a literal
dot on its own line. Blank lines and spaces inside the entry are preserved.
Blank entries are rejected. Closing standard input exits without saving.

The current day is displayed with numbered entries and counts.
Deleting its last entry removes the day and selects the next day, or the
previous day when there is no next day. Cancelling deletion changes nothing.
A failed load leaves the current diary untouched. Existing `DENIK1` files
from the web version are compatible. Save writes and syncs `denik.tmp` before
replacing `denik.txt`.

## How the code works

- `src/main.rs`: terminal prompts, display, command loop, save/load handling.
- `src/model.rs`: date validation, custom lists, diary rules, file format.
- `src/tests.rs`: tests kept separate from the code you present.

Days form a custom singly linked list: `Option<Box<Day>>`. The selected day is
a zero-based index; finding it walks from the head, taking O(number of days).

Each day owns a custom `ArrayList`: linked blocks of eight `Option<String>`
slots. Index 10 is in block `10 / 8 = 1`, slot `10 % 8 = 2`. Appending allocates
a block when needed. Deletion shifts later strings left across block boundaries,
moving ownership rather than cloning strings. Lookups walk the blocks;
this is not a contiguous array with O(1) indexing. Deletion is O(n^2) in the
worst case because it repeatedly finds slots from the head. Empty blocks stay
available for reuse. Both lists are dropped iteratively to avoid deep recursion.

No `Vec`, built-in linked list, or map stores diary data. Comments explain
ownership, index calculations, node insertion/removal, and file parsing.
The length-prefixed file format preserves multiline UTF-8 text.

## Checks

```sh
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

Model tests cover dates, block boundaries, shifting, insertion/deletion rules,
multiline UTF-8 save/load, and malformed files.

## Presentation order

1. Start with `run()` in main.rs: show the day, read a command, perform an action.
2. Explain `add()` and `delete()`: input validation and confirmation.
3. In model.rs, show `Block` and `ArrayList`: arrays linked in blocks of eight.
4. Walk through `get`, `push`, and `remove`, using three entries as an example.
5. Show `Day` and `Diary`: a sorted linked list with a selected day index.
6. Finish with `save` and `load`: byte lengths preserve multiline text.
