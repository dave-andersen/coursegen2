- Run `cargo test` before and after making code changes.
- Run `cargo clippy --all-targets -- -D warnings` before reporting completion.
- Report LoC changes.
- The canonical exercised fixture is `ex/cmu-15712/`. Rebuild it after changes to
  `src/`, `config.toml`, or templates:
  `cargo build && (cd ex/cmu-15712 && ../../target/debug/coursegen2)`.
- `schedule_template.html` is required; `index_template.html` is optional and, when
  present, produces `index.html`. The optional announcements template set produces
  `announcements.html` and `rss2.xml`. Do not edit generated pages or feeds directly;
  change their templates or `config.toml`, then rebuild.
- Templates are runtime-loaded Tera HTML5 templates. Keep derived scalar fields
  autoescaped; only `{{ schedule | safe }}` may render generated HTML.
- `location` is required. `[[instructor]]` records drive both the schedule instructor
  list and the generated index staff table; update config data rather than page markup.
- After generator or template changes, verify the rebuilt example pages in a browser,
  including configured semester, location, instructors, schedule rows, index staff table,
  announcement preview, announcement page, and RSS feed.
- Include regenerated `ex/cmu-15712/schedule.html`, `index.html`, `announcements.html`,
  and `rss2.xml` in the same change when their source config/templates change.
