# claude-statusline-rust

A statusline for [Claude Code](https://claude.com/claude-code). It reads the
session JSON that Claude Code writes to stdin and prints a single powerline
style line to stdout.

```
 claude-statusline-rust ▶ ⎇ main ● ▶ ✱ Opus 5 ▶ ◔ 20,812 (98%) ▶ ◷ 33% ▶ ▦ 4% ▶
```

## Installing

```sh
cargo install --path .
```

Then point Claude Code at it in `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "claude-statusline-rust"
  }
}
```

`cargo install` copies the binary, so re-run it after changing the code.
Statusline commands run per session, so open a new session to see the change.

Flags go in the same string, so to record what Claude Code sends:

```json
"command": "claude-statusline-rust --save-input"
```

To see the output without Claude Code, pipe a session at it:

```sh
claude-statusline-rust < initial-test-data/input.json
```

## Segments

| Segment   | Example            | Shown when                                                       |
|-----------|--------------------|------------------------------------------------------------------|
| directory | ` my-project `     | `workspace.current_dir` is present                               |
| git       | ` ⎇ main ● `       | the directory is inside a git repository                         |
| model     | ` ✱ Opus 5 `      | `model.display_name` is present                                  |
| context   | ` ◔ 20,812 (98%) ` | both `total_input_tokens` and `remaining_percentage` are present |
| five hour | ` ◷ 33% `          | `rate_limits.five_hour.used_percentage` is present               |
| seven day | ` ▦ 4% `           | `rate_limits.seven_day.used_percentage` is present               |

**directory** — the last component of `workspace.current_dir`, so a session in
`/Users/you/code/my-project` shows `my-project`.

**git** — the current branch, or the seven character commit when `HEAD` is
detached. A `●` follows the branch when the working tree has uncommitted
changes, counting untracked files. Read by running `git` in the session
directory rather than from stdin, since Claude Code does not report it.

**model** — `model.display_name` as Claude Code gives it, e.g. `Opus 5`.

**context** — `context_window.total_input_tokens` grouped with commas,
followed by `context_window.remaining_percentage`: how much of the context
window is still free.

**five hour** — `rate_limits.five_hour.used_percentage`, how much of the
rolling five hour limit has been spent. Counts up, unlike the context segment
beside it, which counts down.

**seven day** — `rate_limits.seven_day.used_percentage`, the same for the
seven day window.

Both are shown whenever the session reports them, including at `0%`. Zero is a
value worth seeing rather than an absence, and hiding low numbers would make
the width of the line vary from session to session. The colour is fixed: the
percentage already says how close the limit is, and a background shifting
underneath a changing number is harder to read, not easier.

`resets_at` sits beside each percentage in the session JSON and would allow a
`◷ 33% (2h left)`, but that needs time formatting and a clock, so the bare
percentage is what is shown.

## When something is missing

A statusline renders on every prompt, so it never panics and always exits
zero. A backtrace where the prompt should be is worse than a missing segment.

Every field is optional and each segment appears only when the values behind
it are present. A session without `context_window` still shows the directory,
git and model segments — the rest of the line closes up around the gap.

When the session cannot be parsed at all, or carries nothing worth showing,
the whole line is replaced by a single `⚠ statusline` segment. Nothing is
written to stderr, so to debug a statusline that looks wrong, pipe the JSON
to the binary by hand.

## Saving the input

`--save-input` appends every session read on stdin to
`~/claude-statusline-input.jsonl`, one record per line, before the line is
rendered. Saving first means a session that breaks the renderer is on disk
anyway, which is the case worth having a log for.

It is off by default. A statusline runs on every prompt, so an always on log
grows without anyone asking for it.

Records are normalised so the file stays valid JSONL. A session that parses is
re-serialised compactly, which collapses pretty printed JSON onto one line; a
session that does not parse is saved with its newlines stripped, since
malformed input is exactly what is worth keeping. Empty stdin writes nothing.

Failures are silent, for the same reason the renderer never panics. An
unwritable home directory, a full disk, an unset `$HOME` — the line still
renders and the record is dropped. Opening in append mode means concurrent
sessions add their own lines rather than truncating each other's.

Nothing rotates or prunes the file. It grows until it is deleted by hand.

## Line format

Segments are separated by `U+E0B0`, the powerline right arrow, which needs a
font that provides it. A segment is a background colour, a foreground colour,
and its text padded with one space either side. The separator between two
segments takes the *next* segment's background and the *previous* segment's
foreground; the line ends with a separator whose foreground is the last
segment's background. Colours are 256 colour ANSI codes:

| Segment | Background | Foreground |
|---------|-----------:|-----------:|
| directory | 136 | 231 |
| git | 238 | 231 |
| model | 237 | 231 |
| context | 240 | 251 |
| five hour | 60 | 231 |
| seven day | 66 | 231 |
| warning | 238 | 231 |

60 and 66 are a slate blue and a slate teal. They are near neighbours, so the
two rate limit segments read as one group, and far enough from the 237–240
greys behind model and context for the boundary between them to be visible.

The line runs to 79 columns for this repository on `main`. The model, context
and rate limit segments cost 43 columns between them whatever the session, and
the directory and branch names make up the rest. That fits an 80 column
terminal with a column to spare, so a long branch name, or a directory longer
than this one, is what wraps it.

## Development

```sh
cargo test
```

The tests are outside-in. `tests/statusline_test.rs` holds a golden test that
renders `initial-test-data/input.json` and compares it byte for byte with
`initial-test-data/expected.txt`, which pins the exact escape codes.
`tests/binary_test.rs` runs the built binary over stdin, and
`tests/git_adapter_test.rs` builds throwaway repositories in the temporary
directory to cover clean, dirty, branched and detached states.
`tests/args_test.rs` covers flag parsing and `tests/input_log_test.rs` covers
record normalisation and appending, both against temporary paths.

The binary tests that cover `--save-input` point `$HOME` at a temporary
directory, so running them never writes to the real one.

Git state is injected rather than looked up during rendering: `render` takes
an `Option<&GitStatus>` so tests pass a fake, and `main` supplies the real
adapter. `initial-test-data/output.txt` is the original capture this was
built from and is not used by the tests.

Cost is deliberately not shown. The original capture had a `☉ $1.73 3%`
segment, but neither figure reconciled with the session JSON beside it —
`total_cost_usd` was `0.0887785`, and no field equalled `3` — so it was left
out rather than guessed at.
