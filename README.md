# coursegen2

Generates course pages from `config.toml` and project-local Tera templates: required `syllabus_template.html` and optional `index_template.html`.

Run the generator from the course directory:

```sh
../../target/debug/coursegen2 --config config.toml
```

Both templates are XHTML using [Tera](https://keats.github.io/tera/) syntax. The generator always writes `syllabus.html`; it also writes `index.html` when `index_template.html` exists.

Shared fields:

- `{{ semester }}` — `term` and `year`
- `{{ meeting_times }}` — `meets`, `starts`, and `ends`
- `{{ location }}` — `location`
- `{{ instructors }}` — the `[[instructor]]` records, for Tera loops and conditionals
- `{{ syllabus_instructors | safe }}` — XHTML list derived from each instructor's `name`, `email`, `webpage`, `office`, and `hours`
- `{{ schedule | safe }}` — generated schedule rows, available to the syllabus template
- `{{ generated_at }}` — local generation date

Unknown or unclosed placeholders cause generation to fail. `location` is required in every config.
