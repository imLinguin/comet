#!/bin/bash

# Comet shortcut script
# 
# Meant for usage as a pre-launch script
# Although the script can be used as a wrapper as well
# Heroic Game settings > Advanced > Scripts > Select a script to run before the game is launched  
# Make sure the script is in location that's always accessible by Heroic
# such as /home/deck/Documents

# Variables

gog_username=username
# Full filepath to comet binary - for microSD cards, look into the /run/media/deck/ directory for specific microSD card folders
path_to_comet='/home/deck/Documents/comet/comet'
# Uncomment if debug logs are wanted to be visible in Comet
#export COMET_LOG=debug
# A timeout after which comet will quit when last client disconnects 
export COMET_IDLE_WAIT=5 # comet has a 15 seconds as the default, we make it shorter here, feel free to tweak it to your liking

# Running Comet as a background app 
# If you want to use this script in Lutris change --from-heroic to --from-lutris
exec "$path_to_comet" --from-heroic --username "$gog_username" -q &

# This part allows using this script as a wrapper
# e.g comet_shortcut.sh wine game.exe
if [ $# -ne 0 ]; then
    comet_pid=$!
    exec "$@" &
    game_pid=$!
    echo "Waiting for $comet_pid and $game_pid"
    trap 'kill $game_pid; wait $game_pid $comet_pid' SIGINT SIGTERM
    wait $game_pid $comet_pid
fi

