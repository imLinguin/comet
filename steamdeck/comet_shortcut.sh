#!/bin/bash

# Comet shortcut script
# 
# Meant for Heroic generated non-Steam game shortcuts
# Minor tweaks necessary in the shortcut:
# - Move the launch options (run com.heroicgameslauncher.hgl ...) of the shortcut into the target, next to "flatpak"
# - Put the following in the launch options of the shortcut:
#       "<location of this comet_shortcut.sh>" %command%

# Variables

gog_username=username
# Full filepath to comet binary - for microSD cards, look into the /run/media/deck/ directory for specific microSD card folders
path_to_comet='/home/deck/comet'
# Uncomment if debug logs are wanted to be visible in Comet
#export COMET_LOG=debug

# Running Comet as a separate Konsole window
konsole -e "$path_to_comet --from-heroic --username $gog_username" &

# Grabbing process ID of Comet
comet_pid=$!

# Running the game shortcut under the same process ID as Comet
# Necessary to put Comet and the game in "one opened game" on the Steam Deck's Game Mode 

exec -a "$comet_pid" "$@"