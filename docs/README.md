# Docs Site Development

This directory is published to GitHub Pages with Jekyll and the `just-the-docs` theme.

## Local Preview

From the `docs/` directory:

```bash
bundle install
bundle exec jekyll serve --baseurl "/ez-booth-rs"
```

Then open `http://127.0.0.1:4000/ez-booth-rs/`.

## Notes

- keep published documentation under `docs/`
- prefer relative links between pages inside `docs/`
- link to repository-root documents with full GitHub URLs when the file is not published by Pages
