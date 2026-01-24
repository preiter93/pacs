# pacs

[![Crate IO](https://img.shields.io/crates/v/pacs?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44)](https://crates.io/crates/pacs) ![Crates.io Downloads](https://img.shields.io/crates/d/pacs?style=flat-square) [![Continuous Integration](https://github.com/preiter93/pacs/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/pacs/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/pacs/status.svg?style=flat-square)](https://deps.rs/repo/github/preiter93/pacs) [![License](https://img.shields.io/crates/l/pacs?style=flat-square&color=09bd66)](./LICENSE)

**P**roject **A**ware **C**ommand **S**torage - A CLI for managing and running saved shell commands.

![Demo](pacs/demo/demo.gif)

## Install

```sh
cargo install pacs
```

## Usage

```sh
pacs init                       # initialize pacs
pacs add build "cargo build"    # save a command
pacs run build                  # run it
pacs ls                         # list all commands
pacs edit build                 # edit in $EDITOR
pacs rm build                   # delete it

pacs project add MyProject      # create a project
pacs project switch MyProject   # set active project
pacs project clear              # clear active project
```

## Example Output

```shell
# pacs ls

── Global ──
hello-world
echo "Hello World"

── MyProject ──
get-pods:
kubectl --context dev get pods -o wide
```

## Contexts and Placeholders

Use double curly braces to mark placeholders:
```sh
pacs add get-pods -t k8s 'kubectl --context {{kube-context}} get pods -o wide'
```

Define project-specific contexts and values (for the active project):
```sh
pacs context add dev        # add a context
pacs context edit           # edit contexts in $EDITOR
pacs context ls             # list all contexts
pacs context switch dev     # set active context
```

Listing, running, and copying with a specific context (active project):
```sh
pacs ls -c dev              # list with context
pacs run get-pods -c dev    # run with context
pacs copy get-pods -c dev   # copy with context
```

Notes:
- If no active context is set (or values are missing), pacs shows the raw unexpanded command.
- If active context is set and context values are defined, pacs expands the command before listing, running or copying it.

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
