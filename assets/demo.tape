# DXOS Demo — Terminal Recording
# Run: cd /Users/pran/Projects/dxos && vhs assets/demo.tape

Output assets/demo.gif

Set Shell "bash"
Set FontSize 15
Set Width 880
Set Height 520
Set Padding 16
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set WindowBar Colorful
Set WindowBarSize 40
Set BorderRadius 8
Set CursorBlink false

# Setup: alias dxos to our mock script
Hide
Type "alias dxos='bash /Users/pran/Projects/dxos/assets/fake-dxos.sh'"
Enter
Sleep 200ms
Show

Sleep 500ms

# 1. Show version
Type "dxos --version"
Enter
Sleep 1.5s

# 2. Run a coding task
Type@50ms "dxos run 'fix the null pointer in src/api/handler.rs'"
Enter
Sleep 8s

# 3. Quick init
Type "dxos init"
Enter
Sleep 1.5s

# 4. Show config
Type "dxos config"
Enter
Sleep 3s

Sleep 1s
