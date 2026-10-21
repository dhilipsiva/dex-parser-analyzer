# CLAUDE.md

Read `AGENTS.md` first: it holds the layout, the commands and the rules. In short:

- Work in `nix develop`, and run `just ci` before every commit.
- No panics on malformed DEX input. `docs/rules.md` and `CHANGELOG.md` change together with the code they describe.
- The project's portfolio conventions (licence, deny policy, action pinning, MSRV) are in github.com/dhilipsiva/products, `CONVENTIONS.md`.
