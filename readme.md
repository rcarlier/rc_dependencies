# rc_dependencies

Find dependencies (sub)folders, and their sizes in a given folder.

Usage: rc_dependencies <root_folder> [json_file]

Do not find dependencies folder inside a folder...

## Usage:

```bash
rc_dependencies <root_folder> [json_file]
```

By default, find `["node_modules", ".venv", "venv", ".git"]` but you can define a environnement variable to redefine the list.

Just add in `.bash_profile` (or equiv).

```bash
export RC_DEPENDENCIES=".venv,venv,node_modules,.git"
```

Don't forget to `source ~/.bash_profile` (or equiv).

## Output (json)

Something like this:

```json
{
    "total": 411393588,
    "human": "392.3 MiB",
    "details": [
        {
            "child": "venv",
            "folder": "/Users/richnou/DJANGO/example/back/venv",
            "weight": 66546043,
            "human": "63.5 MiB"
        },
        {
            "child": ".git",
            "folder": "/Users/richnou/DJANGO/example/.git",
            "weight": 27770660,
            "human": "26.5 MiB"
        },
        {
            "child": "node_modules",
            "folder": "/Users/richnou/DJANGO/example/front/node_modules",
            "weight": 317076885,
            "human": "302.4 MiB"
        }
    ]
}
```

## Technical:

```sh
# build exe
cargo build --release

# move to folder in path (MacOS)...
sudo cp ./target/release/rc_dependencies /usr/local/bin/rc_dependencies

```
