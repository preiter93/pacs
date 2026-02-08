# pacs

[![Crate IO](https://img.shields.io/crates/v/pacs?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44)](https://crates.io/crates/pacs) ![Crates.io Downloads](https://img.shields.io/crates/d/pacs?style=flat-square) [![Continuous Integration](https://github.com/preiter93/pacs/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/pacs/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/pacs/status.svg?style=flat-square)](https://deps.rs/repo/github/preiter93/pacs) [![License](https://img.shields.io/crates/l/pacs?style=flat-square&color=09bd66)](./LICENSE)

**P**roject **A**ware **C**ommand **S**torage - A CLI for managing and running saved shell commands.

![Demo](https://raw.githubusercontent.com/preiter93/pacs/main/pacs/demo/demo.gif)

## Install

| Package manager                                        | Command                            |
|--------------------------------------------------------|------------------------------------|
| [Homebrew](https://github.com/preiter93/homebrew-pacs) | `brew install preiter93/pacs/pacs` |
| [Cargo](https://crates.io/crates/pacs)                 | `cargo install pacs`               |

## Usage

```sh
pacs init                       # initialize pacs and create first project
pacs add build "cargo build"    # save a command to active project
pacs run build                  # run it
pacs ls                         # list all commands in active project
pacs edit build                 # edit in $EDITOR
pacs rm build                   # delete it

pacs project add MyProject      # create a project
pacs project switch MyProject   # set active project
pacs project active             # show active project

pacs --ui                       # open the terminal user interface
```

## Example Output

```just
MyProject

hello-world:
echo "Hello World"

[k8s]
get-pods:
kubectl --context dev get pods -o wide
```

## Environments and Placeholders

Use double curly braces to mark placeholders:
```sh
pacs add get-pods -t k8s 'kubectl --context {{kube-context}} get pods -o wide'
```

Define project-specific environments and values:
```sh
pacs env add dev            # add an environment to active project
pacs env edit               # edit environments in $EDITOR
pacs env ls                 # list all environments
pacs env switch dev         # set active environment
```

Listing, running, and copying with a specific environment:
```sh
pacs ls -e dev              # list with environment
pacs run get-pods -e dev    # run with environment
pacs copy get-pods -e dev   # copy with environment
```

Notes:
- All commands are project-scoped. You must have an active project to add or run commands.
- If no active environment is set (or values are missing), pacs shows the raw unexpanded command.
- If active environment is set and environment values are defined, pacs expands the command before listing, running or copying it.

## Shell Completions

**Zsh** (`~/.zshrc`):
```sh
source <(COMPLETE=zsh pacs)
```

**Bash** (`~/.bashrc`):
```sh
source <(COMPLETE=bash pacs)
```

**Fish** (`~/.config/fish/config.fish`):
```sh
source (COMPLETE=fish pacs | psub)
```

## Why PACS?

Why PACS? Why not just use another command runner like `make` or `just`? I use PACS more like a vault than a sophisticated runner. Over time, I’ve accumulated commands that I need every now and then; commands that don’t quite make it into the official scripts, makefiles or justfiles of my projects.  

Even more so when collaborating with others, I don't want to dump all those commands into the repo. But usually the day comes when I have to deploy a busybox or a postgres database in a kubernetes cluster but the docker command is just too difficult for me to remember. Then I find myself scouring through my terminal history to search for it, hoping its still not pushed off the cliff of my history limit...  

I wished I could have all those commands sorted and lying around, ready for the day I need them. That’s why I created PACS. And to be honest, it was also an excuse to write more Rust and to create a new TUI, but that’s still to come.
