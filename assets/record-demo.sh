#!/bin/bash
# Script to record the DXOS demo as an asciinema cast, then convert to GIF
set -e

CAST_FILE="/Users/pran/Projects/dxos/assets/demo.cast"
GIF_FILE="/Users/pran/Projects/dxos/assets/demo.gif"
FAKE_DXOS="/Users/pran/Projects/dxos/assets/fake-dxos.sh"

# Create the script that will be recorded
SCRIPT=$(mktemp)
cat > "$SCRIPT" << 'INNERSCRIPT'
#!/bin/bash

type_slow() {
    local text="$1"
    local delay="${2:-0.04}"
    for ((i=0; i<${#text}; i++)); do
        printf "%s" "${text:$i:1}"
        sleep "$delay"
    done
}

FAKE="/Users/pran/Projects/dxos/assets/fake-dxos.sh"

# Prompt
printf "\033[1;32m❯\033[0m "
sleep 0.3

# Type version command
type_slow "dxos --version"
sleep 0.2
echo ""
bash "$FAKE" --version
sleep 1

# Prompt
printf "\n\033[1;32m❯\033[0m "
sleep 0.3

# Type the main command
type_slow "dxos run 'fix the null pointer in src/api/handler.rs'" 0.035
sleep 0.3
echo ""
bash "$FAKE" run 'fix the null pointer in src/api/handler.rs'
sleep 1.5

# Prompt
printf "\n\033[1;32m❯\033[0m "
sleep 0.3

# Init
type_slow "dxos init"
sleep 0.2
echo ""
bash "$FAKE" init
sleep 1

# Prompt
printf "\n\033[1;32m❯\033[0m "
sleep 0.3

# Config
type_slow "dxos config"
sleep 0.2
echo ""
bash "$FAKE" config
sleep 2

echo ""
INNERSCRIPT

chmod +x "$SCRIPT"

# Record
echo "Recording demo..."
asciinema rec "$CAST_FILE" --overwrite -c "bash $SCRIPT" --cols 88 --rows 28

# Convert to GIF
echo "Converting to GIF..."
agg "$CAST_FILE" "$GIF_FILE" \
    --theme monokai \
    --font-size 16 \
    --speed 1.0 \
    --cols 88 \
    --rows 28

rm "$SCRIPT"
echo "Done! GIF at $GIF_FILE"
ls -lh "$GIF_FILE"
