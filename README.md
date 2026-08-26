# coursegen2

Generates course pages from `config.toml` and project-local Tera templates: required `syllabus_template.html` and optional `index_template.html`.

Run the generator from the course directory; it reads `config.toml` and templates there:

```sh
../../target/debug/coursegen2
```

Pass `--config PATH` to use a different configuration file.


## Project files and outputs

Run from a course directory containing:

- `config.toml`
- `syllabus_template.html` (required)
- `index_template.html` (optional)

Generation writes `syllabus.html` and, when the index template exists, `index.html` in
that same directory. Treat both as generated files: edit their templates or
`config.toml`, then regenerate.

## Templates
Both templates are XHTML using [Tera](https://keats.github.io/tera/) syntax.

Shared fields:

- `{{ semester }}` — `term` and `year`
- `{{ meeting_times }}` — `meets`, `starts`, and `ends`
- `{{ location }}` — `location`
- `{{ instructors }}` — the `[[instructor]]` records, for Tera loops and conditionals
- `{{ syllabus_instructors | safe }}` — XHTML list derived from each instructor's `name`, `email`, `webpage`, `office`, and `hours`
- `{{ schedule | safe }}` — generated schedule rows, available to the syllabus template
- `{{ generated_at }}` — local generation date


## Configuration

`config.toml` requires `year`, `term`, `meets`, `starts`, `ends`, `location`,
`first_day`, `last_day`, `[[instructor]]`, and `[[lecture]]` entries. Dates use
`YYYY-MM-DD`; `meets` accepts weekday names such as `mon` or `monday`.

Each instructor may provide `name`, `email`, `webpage`, `office`, and `hours`.
Each lecture requires `title` and may provide `notes`, `instructor`,
`section_header`, and `[[lecture.papers]]` entries with `title` and `link`.

Optional `[[holiday]]` entries provide a `name` and `dates`; optional
`[[post_class_event]]` entries provide a `date`, `title`, and optional `notes`.

## Scheduled exams

Configure every in-term exam as a top-level TOML array entry:

```toml
[[exam]]
name = "Midterm 1"
date = "2026-10-09"
```

An exam occupies its configured meeting-day slot without consuming a lecture, so the
following lecture remains on its original date.
