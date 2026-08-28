# Code Review — coursegen2

Scope: `src/main.rs`, `Cargo.toml`, root templates/CSS, and the exercised fixture in `ex/cmu-15712/` (config, templates, and generated HTML/CSS/RSS output).

## Critical

### 1. `cargo test` currently fails on `main` — FIXED
`tests::example_configuration_parses_generic_exams` (`src/main.rs:623`) hardcoded the expected exam name as `"Midterm 2, Date Time And Location TBA"`, but `ex/cmu-15712/config.toml:18` was changed to `"Midterm 2: Date, Time, and Location TBA"` in the last commit (`a6c2f2e Polish midterm schedule title`), which updated the config and the regenerated `schedule.html` but not this test.

```
thread 'tests::example_configuration_parses_generic_exams' panicked:
 left: Some([..., ("Midterm 2: Date, Time, and Location TBA", "2026-12-04")])
right: Some([..., ("Midterm 2, Date Time And Location TBA", "2026-12-04")])
```

`AGENTS.md` explicitly requires `cargo test` to pass before/after changes, so this was a real regression left in the tree.

**Fix applied:** the test was rewritten (now `example_configuration_parses`, `src/main.rs:622`) to only assert that the shipped fixture deserializes (`config.is_ok()`), instead of asserting on the exact exam-name/date content. The original test's root problem wasn't just a stale string — it was *structurally* fragile: it pinned test-suite correctness to instructor-authored course content (exam names) that gets "polished" independently of any parsing logic, in a fixture whose whole purpose is to be edited every semester. The behavior it actually needs to guard (exam-array parsing, exam integration with the schedule) is already covered without that coupling by `configured_exam_occupies_its_meeting_slot` (`src/main.rs:528`), which builds its own local `Config`/`Exam` values rather than depending on live fixture text. Verified: `cargo test` → 14 passed, 0 failed; `cargo clippy --all-targets -- -D warnings` → clean.

I audited the rest of the test module for the same failure mode (asserting exact fixture/content text that isn't the thing under test) and found no other instances — every other test builds its own local, in-test data (`test_config()`, inline TOML literals, hand-constructed `Instructor`/`Announcement`/`Exam` values) rather than depending on `ex/cmu-15712/config.toml`'s actual wording, so none of them will break when someone edits course content.

## Bugs

### 2. Exam rows never get the `exam` CSS class — the yellow highlight is dead code — FIXED
`schedule_html` (`src/main.rs:208-221`) renders exam rows as `<tr class="lecture">`, but `ex/cmu-15712/style.css:208` defines `table.schedule tr.exam { background: yellow; }`. Since no code path ever emits `class="exam"`, that rule is unreachable and exams render visually identical to ordinary lectures in the generated `schedule.html` (confirmed: both "Midterm 1" and "Midterm 2" rows come out as `<tr class="lecture">`).

**Fix applied:** exam rows now emit `<tr class="exam">`. Regenerated `ex/cmu-15712/schedule.html` confirms both "Midterm 1" and "Midterm 2" rows now carry `class="exam"` and pick up the existing yellow-highlight rule. Updated the corresponding test (`configured_exam_occupies_its_meeting_slot`, `src/main.rs`) to expect `class="exam"`.

### 3. Duplicate exam dates fail silently instead of erroring (inconsistent with holidays) — FIXED
`holiday_map` (`src/main.rs:126-141`) explicitly detects and errors on duplicate holiday dates. Exams got no such check: `config.exam.iter().map(|exam| (exam.date, exam)).collect::<HashMap<_,_>>()` silently dropped all but the last exam configured for a given date.

**Fix applied:** added `exam_map`, mirroring `holiday_map`'s duplicate-detection pattern — it now returns `Err("duplicate exam date {date}: {name1:?} and {name2:?}")` instead of silently overwriting. `main()` calls it in place of the old inline `.collect()`. New test: `duplicate_exam_dates_fail_with_both_names`.

### 4. Holidays/exams on a non-meeting weekday (or outside the term) vanish silently — FIXED for exams and holidays
The day-by-day loop in `schedule_html` only visits dates that fall on a `meets` weekday between `first_day` and `last_day`, so an exam or holiday dated outside that window (or, for exams, on a non-meeting weekday) previously just disappeared from the generated page with no error.

**Fix applied:** added `validate_schedule`, called from `main()` before generation, which now errors on:
- an exam dated outside `[first_day, last_day]`,
- an exam dated on a weekday not in `meets` (a holiday is *not* required to fall on a meeting weekday — multi-day breaks like the fixture's Fall Break legitimately span non-meeting weekdays, so that check is exam-only),
- a holiday dated outside `[first_day, last_day]`.

New tests: `exam_outside_semester_fails`, `exam_on_non_meeting_day_fails`, `holiday_outside_semester_fails`, plus a positive `well_formed_schedule_passes_validation`.

### 5. Holiday silently wins over a same-day exam (or other scheduled item) with no diagnostic — FIXED, and generalized
`schedule_html` checks `holidays.get(&day)` before `exams.get(&day)`, so a holiday/exam date collision previously just dropped the exam with no indication.

**Fix applied:** `validate_schedule` now errors if any exam or `post_class_event` shares a date with a configured holiday (`"exam {name:?} is scheduled on {date}, which is the holiday {holiday:?}"`, and the equivalent for `post_class_event`). New tests: `exam_on_holiday_fails`, `post_class_event_on_holiday_fails`.

Verified after all of #2-#5: `cargo test` → 21 passed, 0 failed; `cargo clippy --all-targets -- -D warnings` → clean; rebuilt `ex/cmu-15712` fixture (`cargo build && (cd ex/cmu-15712 && ../../target/debug/coursegen2)`) with no validation errors, confirming the real config still passes all the new checks.

## Repo hygiene

### 6. Root `schedule_template.html` is stale, dead content left over from a rename — REMOVED
Was a 122-line monolithic template (tracked, added whole in `d8443c5 "Rename syllabus pages to schedule"`) containing pre-split instructors/textbooks/grading content that has since moved into `ex/cmu-15712/index_template.html`, and never touched again while the real per-course template kept evolving. No `config.toml` at repo root ever exercised it. Deleted (`git rm schedule_template.html`).

### 7. Untracked root `style.css` is stale and out of sync with `ex/cmu-15712/style.css` — REMOVED
Was missing the `.schedule-scroll` wrapper rules, `tr.deadline` background, the responsive media query, and had the old wrong-case `table.Schedule` selector. An untracked stray copy predating the accessibility work. Deleted.

## Minor

### 8. `post_class_event` entries are emitted in config order, not date order
Unlike `announcements`, which are explicitly sorted newest-first (`src/main.rs:168`), post-class events are appended to the schedule in whatever order they appear in `config.toml` (`src/main.rs:289-304`). With only one entry in the current fixture this doesn't show, but multiple post-class events configured out of chronological order would render out of order.

### 9. `clap` is the only unpinned dependency
`Cargo.toml:8`: `clap = { version = "*", features = ["derive"] }` — every other dependency has a real minimum version pinned; `"*"` accepts anything, including a future breaking major version, and is inconsistent with the rest of the file.

### 10. `.gitignore` wasn't updated when announcements/RSS output were added — FIXED
`.gitignore` ignored root-level `/schedule.html` and `/index.html` but not `announcements.html`/`rss2.xml` (added later, in `f50fc51`). Added `/announcements.html` and `/rss2.xml` alongside the existing entries.

### 11. Dead/invalid CSS in `style.css`
- `dl.newslist dt { // display: inline; ... }` — `//` is not a valid CSS comment; was a no-op leftover. **FIXED:** line removed from `ex/cmu-15712/style.css` (the only remaining copy, since the stale root `style.css` was deleted in #7).
- `table.schedule tr.recitation` / `tr.recitation.alt` are defined but never emitted by the generator (there's no "recitation" concept in `src/main.rs`) — dead styling. **Left as-is** (out of scope for this pass).

## Notes (not bugs, but worth being deliberate about)

- Lecture/exam/holiday `title`/`notes`/`name` strings are written straight into the schedule HTML with `write!`/`writeln!` (`src/main.rs:180-307`), bypassing Tera's autoescaping — that's how `notes` fields are allowed to contain raw `<a href=...>` markup (e.g. `config.toml`'s "Optional: `<a href='...'>Chen2023</a>`"). This is fine given `config.toml` is instructor-authored/trusted, and the existing tests (`tera_autoescapes_values_and_allows_explicit_safe_html`) show this was a deliberate choice, but it does mean an accidental `&` or `<` in a plain title/notes field (not intended as HTML) will produce technically-invalid markup rather than an escaped ampersand/bracket. Worth a one-line comment in `schedule_html` noting this is intentionally unescaped, for the next person who touches it.
- RSS `pubDate` always claims `+0000` (UTC) for what's really just a local calendar date (`src/main.rs:149-152`); cosmetic, doesn't affect feed validity.
- Nav links to `exams.html`, `summaries.html`, and `project.html` are hardcoded in every template but not generated by this tool — presumably hand-maintained pages shipped alongside via `ship.sh`. Fine as-is, just flagging that coursegen2 can't tell you if those are missing.

## What's solid

- Test coverage for `schedule_html` is good and reads clearly (holiday-vs-lecture-slot, exam-vs-lecture-slot, excess-lecture rejection, weekday alias parsing).
- Failing closed on excess lectures (`src/main.rs:280-287`) rather than silently truncating the schedule is the right call, and duplicate-holiday detection is a nice touch; `exam_map`/`validate_schedule` now extend that same fail-closed discipline to exams and holiday/exam/post-class-event conflicts (see #3-#5, fixed).
- Optional-template handling (`optional_template`, `src/main.rs:172-178`) cleanly supports the "schedule required, index/announcements/RSS optional" design without duplicating logic.
- `AtomicWriteFile` for all generated outputs avoids partial-write corruption.
