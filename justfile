shebang := '''
    /usr/bin/env bash
    set -euo pipefail
'''

alias r := release
alias s := serve

default:
    just --list

# Populate 'pkg/snapcraft.yaml' and move it to the root.
create-snapcraft:
    #!{{shebang}}

    if [[ "$#" -eq 0 ]]; then
        echo "Usage: just create-snapcraft <TASK>"
        echo ""
        echo "Tasks:"
        echo "  dev: create snapcraft.yaml for dev/edge release"
        echo "  stable: create snapcraft.yaml for stable release"
        exit 2
    fi

    TASK="$1"
    if [[ "$TASK" != "dev" && "$TASK" != "stable" ]]; then
        echo "Invalid task: $TASK"
        exit 3
    fi


    METADATA="$(cargo metadata --format-version=1 --no-deps)"
    VERSION="$(echo $METADATA | jq -r '.packages[0].version')"
    echo "VERSION $VERSION"
    TAGNAME="v$VERSION"
    echo "TAGNAME $TAGNAME"

    TEMPLATE="pkg/snapcraft.template.yaml"
    sed -i "s/<VERSION>/$VERSION/" $TEMPLATE

    if [[ "$TASK" == "stable" ]]; then
        sed -i "s/<GRADE>/stable/" $TEMPLATE
    else
        sed -i "s/<GRADE>/devel/" $TEMPLATE
    fi

    mv --verbose $TEMPLATE snapcraft.yaml

    echo "Created snapcraft.yaml:"
    cat snapcraft.yaml

