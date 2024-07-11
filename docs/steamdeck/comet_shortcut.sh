#!/bin/bash

# Comet shortcut script
# 
# Meant for Heroic's pre-launch script
# Heroic Game settings > Advanced > Scripts > Select a script to run before the game is launched  
# Make sure the script is in location that's always accessible by Heroic
# such as /home/deck/Documents

# Variables

gog_username=username
# Full filepath to comet binary - for microSD cards, look into the /run/media/deck/ directory for specific microSD card folders
path_to_comet='/home/deck/Documents/comet/comet'
# Uncomment if debug logs are wanted to be visible in Comet
#export COMET_LOG=debug
# Uncomment if you want to set a timeout after which comet will close itself if no further connections are established
export COMET_IDLE_WAIT=5 # 15 seconds is the default

# Running Comet as a background app 
exec "$path_to_comet" --from-heroic --username "$gog_username" -q &

