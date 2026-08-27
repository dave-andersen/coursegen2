To rebuild `schedule.html` and `index.html`, run this from this directory:

```sh
../../target/debug/coursegen2
```

Pass `--config PATH` to use another configuration file.

Course-specific static content lives in `schedule_template.html` and `index_template.html`; their dynamic fields come from `config.toml`.
