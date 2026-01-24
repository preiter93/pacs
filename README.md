# pacs

**P**roject **A**ware **C**ommand **S**torage - A CLI for managing and running saved shell commands.

## Install

```sh
cargo install pacs@0.1.0-beta.2
```

## Usage

```sh
pacs add build "cargo build"    # save a command
pacs run build                  # run it
pacs ls                         # list all commands
pacs edit build                 # edit in $EDITOR
pacs rm build                   # delete it

pacs project add myproj         # create a project
pacs project switch myproj      # set active project
pacs project clear              # clear active project
```

## Example Output

```shell
# pacs ls

── Global ──
hello-world
echo "Hello World"

── myproj ──
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
pacs context add dev
pacs context edit
pacs context list
pacs context switch dev
```

Listing, running, and copying with a specific context (active project):
```sh
pacs ls -c dev
pacs run get-pods -c dev
pacs copy get-pods -c dev
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
