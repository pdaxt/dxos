#!/bin/bash
# Record the DXOS v0.3 demo
set -e

CAST="/Users/pran/Projects/dxos/assets/demo.cast"
GIF="/Users/pran/Projects/dxos/assets/demo.gif"
FAKE="/Users/pran/Projects/dxos/assets/fake-dxos-v2.sh"

SCRIPT=$(mktemp)
cat > "$SCRIPT" << 'INNERSCRIPT'
#!/bin/bash
FAKE="/Users/pran/Projects/dxos/assets/fake-dxos-v2.sh"

type_slow() {
    local text="$1"
    local delay="${2:-0.035}"
    for ((i=0; i<${#text}; i++)); do
        printf "%s" "${text:$i:1}"
        sleep "$delay"
    done
}

# Prompt
printf "\033[1;32m❯\033[0m "
sleep 0.5

# Show help
type_slow "dxos --help"
sleep 0.2
echo ""
bash "$FAKE" --help
sleep 1.5

# Explain
printf "\n\033[1;32m❯\033[0m "
sleep 0.3
type_slow "dxos explain" 0.04
sleep 0.2
echo ""
bash "$FAKE" explain
sleep 2

# Fix
printf "\n\033[1;32m❯\033[0m "
sleep 0.3
type_slow "dxos fix" 0.05
sleep 0.3
echo ""
bash "$FAKE" fix
sleep 3

echo ""
INNERSCRIPT

chmod +x "$SCRIPT"

echo "Recording demo..."
asciinema rec "$CAST" --overwrite -c "bash $SCRIPT" --cols 88 --rows 32

echo "Converting to GIF..."
agg "$CAST" "$GIF" \
    --theme monokai \
    --font-size 15 \
    --speed 1.0 \
    --cols 88 \
    --rows 32

rm "$SCRIPT"
echo "Done! GIF at $GIF"
ls -lh "$GIF"
