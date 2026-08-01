# Claude Code Statusline — Plan

A Rust binary that reads Claude Code's session JSON on stdin and writes a
single-line powerline-style statusline to stdout.

## Target output

For the session described by `initial-test-data/input.json`:

```
 initial-test-data ▶ ⎇ main ● ▶ ✱ Opus 5 ▶ § 20.8K tokens ▶ ◔ 20,812 (98%) ▶
```

Byte-for-byte expected output lives in `initial-test-data/expected.txt`.

## Segment layout

The statusline is a sequence of segments separated by `U+E0B0` (powerline
right arrow). A segment is `bg` + `fg` + `" text "`. The separator between
two segments takes the *next* segment's background and the *previous*
segment's foreground; the line ends with a trailing separator whose
foreground is the last segment's background.

| Segment | bg / fg | Content | Source |
|---------|---------|---------|--------|
| directory | 136 / 231 | `initial-test-data` | basename of `workspace.current_dir` |
| git | 238 / 231 | `⎇ main ●` | git port (not stdin) |
| model | 237 / 231 | `✱ Opus 5` | `model.display_name` |
| tokens | 235 / 51 | `§ 20.8K tokens` | `context_window.total_input_tokens` |
| context | 240 / 251 | `◔ 20,812 (98%)` | `context_window.total_input_tokens`, `context_window.remaining_percentage` |

### Formatting rules

- The token count is abbreviated in the tokens segment: `20812` → `20.8K`
  (one decimal place).
- The same count is grouped with commas in the context segment: `20812` →
  `20,812`.
- The percentage in the context segment is `remaining_percentage`, in
  parentheses.
- The git dirty marker `●` is shown only when the working tree has changes.

## Decisions taken

- **Cost is out of scope.** The `☉ $1.73 3%` segment in the original
  `initial-test-data/output.txt` has been dropped. Neither figure reconciled
  with `input.json` (`total_cost_usd` is `0.0887785`, and no field equals `3`),
  and the semantics weren't clear enough to specify. Line 3 is therefore a
  single context segment. `output.txt` is kept as the original capture;
  `expected.txt` is the golden file the tests assert against.
- **Git is a port.** Git state is not present on stdin, so `render` takes it as
  a plain value. Tests pass a fake; `main` wires in an adapter that shells out
  to git in `workspace.current_dir`. This keeps the render path pure and the
  tests deterministic.
- **Outside-in golden test first.** The first test feeds `input.json` through
  the whole render path and asserts the complete output against `expected.txt`.

## Implementation steps (TDD)

Each step is one red-green-refactor cycle.

1. **Golden test for the whole statusline.**
   Failing test: `render(input_json, GitStatus { branch: "main", dirty: true })`
   equals `expected.txt`. Drive out just enough to pass — parse the fields,
   build the segments, join them.

2. **Token abbreviation.**
   Triangulate `20.8K`: cover a value below 1000 (plain digits) and a value in
   the millions, so the rule is pinned by more than the single example.

3. **Comma grouping.**
   Triangulate `20,812`: cover values with zero, one, and two separators.

4. **Git adapter.**
   Test the adapter against a temporary repo: branch name, clean tree, dirty
   tree. Detached HEAD returns the short SHA.

5. **Wire up `main`.**
   Read stdin, call the git adapter with `workspace.current_dir`, print the
   result. Verify end-to-end by piping `input.json` into the binary.

6. **Degrade gracefully.**
   Every field on the session is optional, so a segment is rendered only when
   the value behind it is present. A missing `context_window` costs the token
   and context segments and nothing else.

   When the session cannot be parsed at all, or carries nothing worth showing,
   the statusline is a single `⚠ statusline` segment. Either way the exit
   status is zero and stdout is never a panic message.

   This removed both duplicate parses: `main` parses once to find the
   directory to ask git about, and `render` parses to build the lines.

7. **Collapse onto a single line.**
   The three-line layout came from the original `output.txt` capture and was
   pinned by the golden test from step 1 onwards. Claude Code renders each
   newline as its own row, which is not what the statusline should look like.

   `expected.txt` was regenerated as one line of all five segments, which
   failed the step-1 golden test on the newlines as intended.

   `render_lines` had appended `"\n"` after every line. The segment lists are
   now concatenated and rendered as a single line, with one trailing newline
   on the output.

   The three builders were renamed to `directory_segments`, `model_segments`
   and `context_segments`: they contribute segments to the line rather than
   producing lines of their own. Each still owns its segments, so the
   step-6 behaviour is unchanged — a missing `context_window` drops its
   segments from the middle of the line rather than removing a row.

   The line is 76 columns wide for the example session. A narrower terminal
   will wrap it, which would look like this problem returning. Dropping the
   `§ 20.8K tokens` segment would bring it to about 55, since it repeats the
   count already shown by `◔ 20,812`.

## Dependencies

- `serde` + `serde_json` for parsing the input. The input JSON has many fields
  we ignore, so the input struct only declares the ones listed above.
