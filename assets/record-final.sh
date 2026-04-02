#!/bin/bash
set -e

CAST="/Users/pran/Projects/dxos/assets/demo.cast"
GIF="/Users/pran/Projects/dxos/assets/demo.gif"

SCRIPT=$(mktemp)
cat > "$SCRIPT" << 'INNERSCRIPT'
#!/bin/bash
cd /Users/pran/Projects/dxos

type_slow() {
    for ((i=0; i<${#1}; i++)); do
        printf "%s" "${1:$i:1}"
        sleep 0.04
    done
}

printf "\033[1;32m❯\033[0m "
sleep 0.4
type_slow "dxos --version"
echo ""
dxos --version
sleep 0.8

printf "\n\033[1;32m❯\033[0m "
sleep 0.3
type_slow "dxos run 'what files are in this project' --model qwen3:8b"
echo ""
dxos run "what files are in this project" --model qwen3:8b 2>&1
sleep 1.5

echo ""
INNERSCRIPT

chmod +x "$SCRIPT"

echo "Recording..."
asciinema rec "$CAST" --overwrite -c "bash $SCRIPT" --cols 90 --rows 28

echo "Converting at 1x speed..."
agg "$CAST" "$GIF" \
    --theme monokai \
    --font-size 15 \
    --speed 1.0 \
    --cols 90 \
    --rows 28

rm "$SCRIPT"
echo "Done!"
ls -lh "$GIF"
