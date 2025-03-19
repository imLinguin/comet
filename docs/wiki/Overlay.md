# GOG Galaxy Overlay

This section describes how to use Galaxy overlay with comet.

> [!IMPORTANT]
> At the moment, overlay injection is implemented only on Linux with galaxy-helper.
> Windows compatibility is a matter of time. While MacOS is not planned in forseable future.

## Step by step

- Get [galaxy-helper](https://github.com/imLinguin/galaxy-helper). Unpack it to your location of choice
- Download / Update overlay
  ```
  comet --from-heroic --username <username> overlay --force
  ```
- `HEROIC_APP_NAME` environment variable allows comet to load metadata for given app and give context to overlay itself. (this is required for welcome popup and in-game invites)
- Start comet, at least v0.3.0
- Run the game

## Running the game

Below is the example with UMU. Since pressure vessel shares `/tmp` with host we can ensure the pipes are shared between contaier runtime.

```
GAMEID=0 STEAM_COMPAT_INSTALL_PATH=/game/install/location umu-run galaxy.exe game.exe
```

### Breakdown

- STEAM_COMPAT_INSTALL_PATH - is a install location that galaxy-helper uses to determine game details. This path is also used by UMU to ensure it is available in the container runtime.
- GAMEID - standard variable for umu. Used to automatically apply required workarounds for games.
- umu-run - umu entry point
- galaxy.exe - main executable of galaxy-helper. It should be located together with its `libgalaxyunixlib.dll.so`. The exe doesn't need to be in your current directory - just ensure to provide full path to its location.
- game.exe - normal command you'd run to start the game.

## Doing this in Heroic (LINUX)
Until overlay support ships in Heroic itself its still possible to add injection code.  
You need to create custom wrapper script that will modify the launch command to include path to `galaxy.exe` before game executable itself. Make sure the executable is in location that is accessible from within umu container.

```bash
#!/bin/bash

# Static path to insert
INSERT_PATH="/path/to/galaxy.exe"

NEW_ARGS=()
PREV_MATCHED=false

for arg in "$@"; do
    if [[ "$PREV_MATCHED" == true ]]; then
        NEW_ARGS+=("$INSERT_PATH")
        PREV_MATCHED=false
    fi
    NEW_ARGS+=("$arg")
    if [[ "$arg" == *"umu-run" || "$arg" == *"umu-run.py" ]]; then
        PREV_MATCHED=true
    fi
done

# If the last argument was umu-run or umu-run.py, add the path at the end
if [[ "$PREV_MATCHED" == true ]]; then
    NEW_ARGS+=("$INSERT_PATH")
fi

# Print the new command for debugging
echo "Modified command: " "${NEW_ARGS[@]}"

# Execute the modified command
exec "${NEW_ARGS[@]}"


```

## Current limitations

- Game invitations may not work
- Success of overlay working is bound if comet was able to read up-to date token information.
- Overlay itself is limited to work only when online. It relies on its online services for most features.

## Technical details

How does the galaxy-helper work? Why is it required?

First we need to understand how communication with overlay occurs. Upon starting the overlay, it gets game pid it needs to attach to via an argument. This information is also used for IPC, overlay connects to two pipes.

- `\\.\pipe\Galaxy-{pid}-CommunicationService-Overlay`
- `\\.\pipe\Galaxy-{pid}-Overlay-Game`

The first one is crucial for communication with what acts as a Galaxy client, in this case comet. Since those pipes are only visible in Wine and comet runs natively on Linux, we need a way to communicate these two. That's where galaxy-helper comes in.

When you start the galaxy.exe it

- scans `STEAM_COMPAT_INSTALL_PATH` for game details
- looks for game executable to be available
- when executable is found it injects the overlay and contacts comet to let it know about the pid
- after that galaxy-helper communicates `\\.\pipe\Galaxy-{pid}-CommunicationService-Overlay` and `/tmp/Galaxy-{pid}-CommunicationService-Overlay` together.

### Why the weird /tmp location

It mostly comes down on what the client libraries shipped with games are ready for. Since their IPC code is generic for all unix OS, it follows the same pattern as it would on Mac. If we ever get a native overlay for Linux, it would communicate through `/tmp/Galaxy-{pid}-Overlay-Game` and the game itself would expect it there.
