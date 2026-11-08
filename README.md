# Schemify-Git

Version control for [Schemify](https://github.com/Scouren), the schematic editor for analog and mixed-signal IC design. Schemify has no version control of its own: each system is a plugin, and this repository holds the first-party ones.

| Plugin | VCS | In Schemify |
|---|---|---|
| `git` | git | built in, on by default |
| `hg` | Mercurial | built in, off until you enable it |
| `jj` | Jujutsu | built in, off until you enable it |
| `svn` | Subversion | built in, off until you enable it |
| `p4` | Perforce Helix Core | built in, off until you enable it |

Each one gives a project:

- **Source control pane:** the project's changed files, each with a letter: `M` modified, `A` added, `D` deleted, `R` renamed, `U` untracked. Click a row to open its diff.
- **Marks in the Explorer:** the same letter after each changed file.
- **Diffs:** a design file (`.shcomp`, `.shprim`, `.shbench`) gets a diff object by object. Any other file gets its text diff.
- **Who changed each line,** for the blame view (the `vcs.annotate` convention in Schemify's `docs/plugins/vcs.md`).
- **`git.setup`** (git only): sets up Schemify's semantic merge driver in the clone and writes the `.gitattributes` lines for design files. Commit `.gitattributes` so every clone shares it.

Commits, branches and checkouts stay yours, in a terminal or your VCS's own tools. A plugin never changes the repository.

## Install

Nothing to install: Schemify ships these plugins inside the app.

- **git** is on by default. To turn it off, open **Packages** and switch it off.
- **The others** are listed on the store's Plugins tab under **Version control**. Enable the one your project uses. A project uses one version-control plugin at a time.
- **The program must be on `PATH`:** `git`, `hg`, `jj`, `svn` or `p4`. The plugin runs it with Schemify's `spawn:<program>` permission.

To try a change before it ships, build the modules and add one as a path package in your `init.lua`:

```sh
nix develop -c ./build.sh          # crates/<name>/<name>.wasm
```

```lua
schemify.pack.add{ { path = "/path/to/Schemify-Git/crates/hg", grant = { "spawn:hg", "read_project", "write_project", "commands" } } }
```

## Layout

| Path | What it is |
|---|---|
| `crates/<name>` | One plugin: its `schemify.project`, its committed `<name>.wasm` and a few lines of Rust that run the shared driver over its VCS row. |
| `crates/git/tests/real_git.rs` | The git row against real git: status, diff, show, log and annotate in a temporary repository. |
| `build.sh` | Builds every module (wasm32, reproducible in `nix develop`). |

The shared code lives in the Schemify SDK, [Scouren/Schemify-Plugins](https://github.com/Scouren/Schemify-Plugins), as the `schemify-vcs` crate (`sdk/rust/vcs`):

- the backend rows and their parsers, each tested on its tool's documented output;
- the pane;
- the annotate file;
- the driver.

To add a VCS of your own, follow the SDK's `docs/plugins/vcs.md`.

## Test

```sh
nix develop -c cargo test           # native: the parsers and the real-git tests
nix develop -c cargo clippy --all-targets -- -D warnings
nix develop -c ./build.sh           # then commit the changed crates/*/*.wasm
```

## Licence

MIT, see [LICENSE](LICENSE).
