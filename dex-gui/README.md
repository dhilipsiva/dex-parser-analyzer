# dex-gui

The in-browser front end of [dex-parser-analyzer](../README.md): a client-side Leptos app, compiled to WebAssembly, that parses a `.dex` file with `dexpa-core`, runs every `dexpa-analysis` pass and shows the overview and the findings. The file is read in the page and never uploaded. Live at https://dhilipsiva.github.io/dex-parser-analyzer/.

## Local development

```bash
nix develop
cd dex-gui && trunk serve
```

Open `http://localhost:8080` and pick a `.dex` file. The dev shell provides trunk, the wasm32 target and everything else (trunk 0.21 from the flake's nixpkgs).

## The release bundle

`nix build .#dex-gui` builds what GitHub Pages serves: `index.html`, the JS glue, the wasm module and two stylesheets, all with relative URLs, so the bundle works under any path.

- Nothing loads from a third party. Bootstrap 5.3.3's CSS is vendored in `assets/` (MIT; byte-identical to the release file, SRI `sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH`), and `assets/app.css` holds the three styles Bootstrap lacks.
- A release build carries a Content-Security-Policy `<meta>` tag: `default-src 'none'`, scripts from the page's origin plus trunk's inline loader by hash, `'wasm-unsafe-eval'` to compile the module, and styles from the origin only, so the view code uses classes, never `style` attributes. `csp.py`, a trunk `post_build` hook, writes it, because the loader's hash is known only after trunk has written it. `trunk serve` builds are left without a policy, so trunk's autoreload keeps working.

`.github/workflows/deploy-dex-gui.yml` builds the bundle with the flake and deploys it on every push to `main` that touches the GUI, `dex-core`, `dex-analysis`, the lockfiles or the toolchain pins, and after each release.

## Tests

`nix develop .#wasm-test -c just test-wasm` runs the browser smoke tests in `src/smoke.rs` with wasm-bindgen-test in headless Chromium: the app mounts, and the analyzer, compiled to wasm32, reports and renders the findings of the rule fixture. CI runs them in the `gui-smoke` job.
