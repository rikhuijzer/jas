shebang := '''
    /usr/bin/env bash
    set -euo pipefail
'''

default:
    just --list

set positional-arguments

# Populate 'pkg/snapcraft.yaml' and move it to the root.
create-snapcraft TASK:
    #!{{shebang}}

    TASK="{{TASK}}"
    if [[ "$TASK" != "dev" && "$TASK" != "stable" ]]; then
        echo "Invalid task: $TASK"
        exit 3
    fi

    # Check if sd is installed
    if ! command -v sd &> /dev/null; then
        echo "Error: 'sd' command not found."
        echo "This script uses 'sd' since it is more platform agnostic than 'sed'."
        exit 1
    fi

    METADATA="$(cargo metadata --format-version=1 --no-deps)"
    VERSION="$(echo $METADATA | jq -r '.packages[0].version')"
    echo "VERSION: $VERSION"
    TAGNAME="v$VERSION"
    echo "TAGNAME: $TAGNAME"

    TEMPLATE="pkg/snapcraft.template.yaml"
    echo "TEMPLATE: $TEMPLATE"
    sd "<VERSION>" "$VERSION" "$TEMPLATE"

    if [[ "$TASK" == "stable" ]]; then
        sd "<GRADE>" "stable" "$TEMPLATE"
    else
        sd "<GRADE>" "devel" "$TEMPLATE"
    fi

    cp --verbose "$TEMPLATE" snapcraft.yaml

    echo "Created snapcraft.yaml:"
    cat snapcraft.yaml

