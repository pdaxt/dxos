#!/bin/bash
# Record REAL dxos output — no fakes
set -e

CAST="/Users/pran/Projects/dxos/assets/demo.cast"
GIF="/Users/pran/Projects/dxos/assets/demo.gif"

SCRIPT=$(mktemp)
cat > "$SCRIPT" << 'INNERSCRIPT'
#!/bin/bash

type_slow() {
    local text="$1"
    local delay="${2:-0.035}"
    for ((i=0; i<${#text}; i++)); do
        printf "%s" "${text:$i:1}"
        sleep "$delay"
    done
}

cd /Users/pran/Projects/dxos

printf "\033[1;32m❯\033[0m "
sleep 0.3
type_slow "dxos --version"
echo ""
dxos --version
sleep 1

printf "\n\033[1;32m❯\033[0m "
sleep 0.3
type_slow "dxos run 'list all Cargo.toml files and count them'" 0.03
echo ""
dxos run "list all Cargo.toml files and count them" --model qwen3:8b 2>&1
sleep 2

echo ""
INNERSCRIPT

chmod +x "$SCRIPT"

echo "Recording REAL demo..."
asciinema rec "$CAST" --overwrite -c "bash $SCRIPT" --cols 90 --rows 30

echo "Converting to GIF..."
agg "$CAST" "$GIF" \
    --theme monokai \
    --font-size 15 \
    --speed 1.5 \
    --cols 90 \
    --rows 30

rm "$SCRIPT"
echo "Done!"
ls -lh "$GIF"
