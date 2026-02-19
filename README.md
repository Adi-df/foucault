# Foucault
A small terminal UI note-taking app.

![Demo](doc/demo.gif)

Note : Demo made using the [Helix](https://github.com/helix-editor/helix) editor for editing notes and scraped data from Wikipedia to fill the notebook.

# Install Foucault

## Shell Installer

Thanks to [cargo-dist](https://github.com/axodotdev/cargo-dist), an installer script exists to install foucault with just one command.

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Adi-df/foucault/releases/download/v0.3.5/foucault-installer.sh | sh
```

## Binaries

Look through [Releases](https://github.com/Adi-df/foucault/releases) for binaries and MSI installers.

## Building from source

The easiest way to build foucault from source is to use the [just](https://github.com/casey/just) command runner.

```sh
# Clone the foucault repo
git clone https://github.com/Adi-df/foucault
# Use build-release to build using optimisation
just build-release
# Use prepare-queries and build-dev to build using the dev profile
just prepare-queries
just build-dev
```

# Usage

## Creating your first notebook

Foucault is based on the notion of Notebook which contains notes, tags, etc.
It exposes a CLI app made with [clap](https://github.com/clap-rs/clap) to manage notebooks.
To create your first notebook, use `foucault create [NAME]`.
Then open it with `foucault open [NAME]`.

## Using foucault to take notes

The keymap is detailed when toggling the help bar with `CTRL+H`.

Editing notes works with an external editor set by the `EDITOR` env variable, so that you may use your favorite editor.
Notes are taken in Markdown with (limited) support. It supports making cross-references between notes by using `[[NOTE_NAME]]`.

## Exposing a notebook / Connecting to one

Foucault supports accessing remote notebooks : Expose the notebook through `foucault serve [NAME]`. And connect to it with `foucault connect http://remotenotebookadress.org`.

# Is it any good ?

Probably not.

Foucault is just a side project I made to take notes in philosophy courses.
There are probably plenty of bugs, it's inefficient and missing a lot of features.
But it still kinda works, so maybe take a look !

# Built with

  - The fantastic Rust language.
  - The smashing [clap](https://github.com/clap-rs/clap) command parser.
  - The amazing [Tokio](https://github.com/tokio-rs/tokio) async runtime.
  - The wonderful [SQLite](https://www.sqlite.org/) database through the brilliant [SQLx](https://github.com/launchbadge/sqlx) library.
  - The awesome [axum](https://github.com/tokio-rs/axum) web framework.
  - The incredible [ratatui](https://github.com/ratatui-org/ratatui) TUI library.
  - The terrific [just](https://github.com/casey/just) command runner.
  - The superb [cargo-dist](https://github.com/axodotdev/cargo-dist) app packager.
  - And many other excellent open source crate.
