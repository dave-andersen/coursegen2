- Run `cargo test` before and after making code changes.
- Run `cargo clippy --all-targets -- -D warnings` before reporting completion.
- Report LoC changes.
- The canonical exercised fixture is `ex/cmu-15712/`. Rebuild it after changes to
  `src/`, `config.toml`, or templates:
  `cargo build && (cd ex/cmu-15712 && ../../target/debug/coursegen2)`.
- `syllabus_template.html` is required; `index_template.html` is optional and, when
  present, produces `index.html`. Do not edit generated `syllabus.html` or `index.html`
  directly; change the corresponding template or `config.toml`, then rebuild.
- Templates are runtime-loaded Tera XHTML templates. Keep derived scalar fields
  autoescaped; only `{{ schedule | safe }}` and `{{ syllabus_instructors | safe }}`
  may render generated XHTML.
- `location` is required. `[[instructor]]` records drive both the syllabus instructor
  list and the generated index staff table; update config data rather than page markup.
- After generator or template changes, verify the rebuilt example pages in a browser,
  including configured semester, location, instructors, schedule rows, and index staff table.
- Include regenerated `ex/cmu-15712/syllabus.html` and `ex/cmu-15712/index.html` in
  the same change when their source config/templates change.
