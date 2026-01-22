# pacs

**P**roject **A**ware **C**ommand **S**torage - A CLI for managing and running saved shell commands.

## Install

```sh
cargo install --path pacs
```

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

## Usage

```sh
pacs add build "cargo build"    # save a command
pacs run build                  # run it
pacs list                       # list all commands
pacs edit build                 # edit in $EDITOR
pacs remove build               # delete it

pacs project add myproj         # create a project
pacs project activate myproj    # set active project
```
