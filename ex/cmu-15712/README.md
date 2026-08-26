To rebuild `syllabus.html` and `index.html`, run this from this directory:

```sh
../../target/debug/coursegen2 --config config.toml
```

Course-specific static content lives in `syllabus_template.html` and `index_template.html`; their dynamic fields come from `config.toml`.
