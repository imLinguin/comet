# Using with the Steam Deck

Using Comet with Valve's Steam Deck (running SteamOS) is possible in both Desktop and Game Mode. Comet will only function with games that do support [GOG Galaxy's achievement system](https://www.gog.com/en/games?features=achievements) or any other online related functionality like Leaderboards. **If your game does not work out of the box, check Known Issues below.**

Using `comet_shortcut.sh` will simplify the process of launching Heroic Games Launcher games with Comet in the background. That script works in both Modes.

This instruction also applies for desktop users who want to use comet in launchers that do not support comet natively.


## Installation steps

1. Make sure you are logged into GOG on your launcher of choice.
2. Download the latest release of Comet from [the latest release](https://github.com/imLinguin/comet/releases/latest) or [the latest GitHub Actions run](https://github.com/imLinguin/comet/actions/workflows/build.yml) labelled `comet-x86_64-unknown-linux-gnu` (a 64bit executable for Linux).
3. Extract the downloaded archive to a desired place.
   > It is recommended to have the `comet` binary put into the `~/Documents` directory. Otherwise: choose any directory where Heroic has access to.
4. Open the `comet_shortcut.sh` file with Kate (right click on the file > Open with Kate), and edit the following values:
    - `gog_username`
        > Change the `username` value after `=` to your GOG username. If your name includes any special characters make sure to quote the username accordingly
    - `path_to_comet`
        > Change the `path_to_comet` value after `=` (while keeping the `'` characters in tact) to the full file path of the `comet` binary.
5. Start any game that has the shortcut script included (see instructions for [Heroic](#use-with-heroic) or [Lutris](#use-with-lutris)) to play the game with achievement support!

> [!NOTE]
> On the startup comet downloads `GalaxyPeer` libraries (~100 MiB) into `$XDG_DATA_HOME/comet/redist/peer`.
> The download is triggered if there is an update available or if the files aren't downloaded already. 
> On Mac, both native and windows libraries are being downloaded


## Use with Heroic 
1. Install Comet and its shortcut script. (See the installation steps above.)
2. In Heroic Games Launcher, set the `comet_shortcut.sh` as a script that is going to be ran before the game (Game Settings > Advanced > Scripts)
4. Launch the game through either Desktop or Game Mode!
5. **Directly exiting the game through Steam will not sync your GOG playtime via Heroic!** Make sure to always exit the game via in-game menu.

> [!TIP] 
> Use Heroic's Add to Steam feature for the best experience
> in accessing your games from the Gaming Mode

## Use with Lutris

If you want to use the same script with Lutris, modify it to include `--from-lutris` instead of `--from-heroic`. You need to be logged in the GOG runner in Lutris.

Steps to add a script to Lutris game

1. Install Comet and its shortcut script. (See the installation steps above.)
2. Right click on the game and click Configure
3. Head over to `System options` and enable `Advanced` mode (next to save button)
4. Scroll down into `Game execution` section and set the path to `comet_shortcut.sh` as Command prefix or Pre-launch script.
5. Launch the game normally!

> [!TIP] 
> Setting the script as Command prefix will allow Lutris to wait for both comet and game process.
> Thus it's a recommended way

## Known Issues

- **Not all GOG games are supported out of the box** - some games (e.g. [Cuphead](https://www.gog.com/game/cuphead)) do not support the way Comet works on its own, due to some checks performed by SDK used for GOG Galaxy features. 
  
  **To solve it**: you will need to install the `GalaxyCommunications` dummy application. (The necessary files (the `.bat` script and the dummy `.exe`) have been included in the archive together with comet binary.)

  1. Keep the `comet` Linux artifact items in a directory Heroic has access to, such as `~/Documents` or `~/Desktop`.
  2. Go to Heroic Games Launcher, to the malfunctioning game's settings screen..
  3. Scroll down the WINE tab of the game's settings screen until you see `RUN EXE ON PREFIX`.
  4. Drag and drop the `install-dummy-service.bat` onto `RUN EXE ON PREFIX` to install the dummy service for the game to detect.
  5. Play the game as you would expect. It should now function with Comet's features!

- Currently there is no visible feedback on when the achievement has been unlocked, there is a [decky-loader plugin planned](https://github.com/imLinguin/comet/issues/18)

## Offline support

Comet should be able to register achievements and statistics while offline just fine and report them to the server next time you play the game online.  

**Please make sure to report issues if you encounter any.**

