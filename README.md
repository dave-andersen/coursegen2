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
- `announcements_template.html`, `announcements_list.html`, `announcements_preview.html`, and
  `rss_template.xml` (optional announcement-page set)

Generation writes `syllabus.html` and, when the corresponding optional templates exist,
`index.html`, `announcements.html`, and `rss2.xml` in that same directory. Treat generated
pages and feeds as output: edit their templates or `config.toml`, then regenerate.

## Templates
Both templates are HTML5 using [Tera](https://keats.github.io/tera/) syntax.

Shared fields:

- `{{ semester }}` — `term` and `year`
- `{{ meeting_times }}` — `meets`, `starts`, and `ends`
- `{{ location }}` — `location`
- `{{ instructors }}` — the `[[instructor]]` records, for Tera loops and conditionals
- `{{ schedule | safe }}` — generated schedule rows, available to the syllabus template
- `{{ generated_at }}` — local generation date
- `{{ announcements }}` — every configured announcement plus the automatic first-day notice
- `{{ recent_announcements }}` — the two newest announcements, for the index preview


## Configuration

`config.toml` requires `year`, `term`, `meets`, `starts`, `ends`, `location`,
`first_day`, `last_day`, `[[instructor]]`, and `[[lecture]]` entries. Dates use
`YYYY-MM-DD`; `meets` accepts weekday names such as `mon` or `monday`.

Each instructor may provide `name`, `email`, `webpage`, `office`, and `hours`.

Configure the optional course secretary with:

```toml
[course_secretary]
name = "Emi Perdan"
email = "eperdan@cs.cmu.edu"
```
Each lecture requires `title` and may provide `notes`, `instructor`,
`section_header`, and `[[lecture.papers]]` entries with `title` and `link`.

Optional `[[holiday]]` entries provide a `name` and `dates`; optional
`[[post_class_event]]` entries provide a `date`, `title`, and optional `notes`.
Holiday dates must be unique across every holiday entry; duplicates fail generation and name both conflicting holidays.

## Announcements

The generator always adds an announcement for the configured `first_day`. Add later
announcements with top-level TOML entries:

```toml
[[announcement]]
date = "2026-09-01"
title = "Office hours updated"
body = "See the course staff page."
```

Announcements are ordered newest first. The full page uses
`announcements_template.html`; `announcements_preview.html` is included by the index;
`rss_template.xml` generates the static `rss2.xml` feed.


## Scheduled exams

Configure every in-term exam as a top-level TOML array entry:

```toml
[[exam]]
name = "Midterm 1"
date = "2026-10-09"
```

An exam occupies its configured meeting-day slot without consuming a lecture, so the
following lecture remains on its original date.
