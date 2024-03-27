# Using with the Steam Deck

Using Comet with Valve's Steam Deck (running SteamOS) is possible in both Desktop and Game Mode. Comet will only function with games that do support [GOG Galaxy's achievement system](https://www.gog.com/en/games?features=achievements).

Using `comet_shortcut.sh` will simplify the process of launching Heroic Games Launcher generated non-Steam game shortcuts with Comet in the background. That script works in both Modes.

## Known Issues

- **Not all GOG achievement games are supported** - some games (e.g. [Cuphead](https://www.gog.com/en/game/cuphead)) do not support the way Comet currently works. Your milage may vary in game support. See (and contribute!) to the Compatibility Chart seen in Comet's wiki.

## Installation steps

1. Make sure you are logged into GOG on Heroic Games Launcher.
2. Download the latest release of Comet from [the latest GitHub Actions run](https://github.com/imLinguin/comet/actions) labelled `comet-x86_64-unknown-linux-gnu`.
3. Extract the downloaded `.zip` archive to a desired place.
   > It is recommended to have the `comet` binary put into the `/home/deck/` directory, as that is the default file path in the script. You are free to change it to a different install location, however. This does include on external storage such as a microSD card inserted in the Steam Deck.
4. Download `comet_shortcut.sh` from the GitHub repository. (Click on the file, then on the icon with the arrow pointing downwards into the bracket.)
5. Put the `comet_shortcut.sh` file in any location.
   > It is recommended to put it in the same directory as the `comet` binary.
6. Mark both `comet_shortcut.sh` and `comet` as `is Executable`. (Right click on the file > Properties > Permissions tab > click on the `Is executable` checkbox.)
7. Use Heroic Games Launcher to create non-Steam game shortcuts of any of your GOG games to use with Comet.
8. Restart Steam to have the newly created shortcuts show up.
9. For each Heroic Games Launcher shortcut:
    - Move the current launch options (`run com.heroicgameslauncher.hgl ...`) into the target field, next to `flatpak`.
    - Put the following in the launch options field:
    `"/home/deck/comet_shortcut.sh" %ccommand%`
        > If you put the `comet_shortcut.sh` file in a different location: replace `/home/deck/comet_shortcut.sh` with the full file path of `comet_shortcut.sh`.
10. Open the `comet_shortcut.sh` file with Kate (right click on the file > Open with Kate), and edit the following values:
    - `gog_username`
        > Change the `username` value after `=` to your GOG username
    - `path_to_comet`
        > Change the `path_to_comet` value after `=` (while keeping the `'` characters in tact) to the full file path of the `comet` binary.
11. Start any game shortcut that has the shortcut script included (see step 9.) in its launch options to play the game with achievement support!
