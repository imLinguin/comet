# Game Compatibility

As this is a work-in-progress replacement, not all GOG Galaxy feature enabled games may function as expected. You can use `CTRL+F` to search for the game you would like to learn more about its compatibility.

## Contribute

To contribute, follow the Contribution Guidelines mentioned in the [Home Page of the Wiki](Home) and follow the following steps:

<details>
<summary>Steps</summary>

- Copy the last column of the table
- Edit the following information for your tested game compatibility:
  - Game Title (with the GOG storefront linked to it as a Hyperlink, in the following format: `[Game's name](GOG url here)`)
  - Game Version
    - To check the official Game Version name, either find it in-game or follow the following steps on Heroic:
      1. Go to the game's page on Heroic Games Launcher
      2. Hover over the three-point menu button and click on `Modify Installation`
      3. Click on the checkbox next to `Keep the game at specific version`
      4. The selected version should be the one you have currently installed. Note the version and date of said version as Game Version.
  - If the Native Linux Version works with Comet
      - Some games that do have a native Linux version available (not using Proton/WINE), still contain code for GOG features that remain unused unless used with Comet. If the game you tested does have a native Linux version, please do test if the Linux version does connect with Comet. 
  - Comet Version
    - Go by the version name in the Releases tab, of the version you downloaded.
    - Did you use Comet when it had no Releases available, or are you using a build that's not part of the releases? Mention (and possibly link) to the commit of the version you used.
  - GalaxyCommunication.exe Service Required
    - Test if the game does function with achievements and leaderboards/Comet. If it does not, try to install the service as you can [read in the dummy service documentation]](https://github.com/imLinguin/comet/tree/main/dummy-service/README.md).
    - If the service is required, fill in with `🟩 Yes`. If the service is not required, fill in with `🔲 No`.
  - GOG Galaxy Features
    - Fill in the `🟩` (working), `🔲` (not present in-game) or `❌` (not working) for the following features:
      - Achievements
      - Leaderboard
  - Notes
    - Any additional notes you would find important to mention with regards to the game compatibility. For example: possible issues, workarounds, specifics like switchable leaderboards between GOG and a different service.
</details>

## Game Compatibility Table

### Legend


| Icon | Meaning |
|---|---------------------------------------------------------------------------------------|
| 🟩 | Required (GalaxyCommunication.exe requirement), Working (GOG Galaxy Features)         |
| 🔲 | Not Available (Native Linux Version, GOG Galaxy Feature), Not Required (GalaxyCommunication.exe requirement)
| ❌ | Not Working (GOG Galaxy Features)|
| ❓| Unknown (GalaxyCommunication.exe requirement; in case of game-specific bugs that prevent Comet/GOG connection)

About `Native Linux Version Works`:

While GOG does not officially support GOG Galaxy features (achievements and leaderboards) for Linux versions of GOG games due to the lack of a Linux version of GOG Galaxy, some games that are multi-platform (having Windows and/or macOS versions, besides a native Linux version) still do have code in the Linux version to enable GOG Galaxy features. This connection is unused for Linux versions, but can get used by Comet. **Not all Linux versions do ship with unused/leftover GOG Galaxy connecting code, however.** Do not expect every native Linux version to work with Comet.

### Table

|Game Title|Game Version|Native Linux Version Works|Comet Version|GalaxyCommunication.exe Service Required|GOG Galaxy Features|Notes|
|-----|-----|-----|-----|-----|-----|-----|
|[Absolute Drift](https://www.gog.com/game/absolute_drift)|5f6049d (6/26/2023)|🔲 Not Available|[commit `ed38c3d`](https://github.com/kevin-wijnen/comet/commit/ed38c3d5253893779ba3d7ab828af442652f6044)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements do work. Leaderboard support works as of Comet version `ed38c3d`.|
|[Alder's Blood Prologue](https://www.gog.com/game/alders_blood_prologue)|1.0.20a (4/13/2020)|🔲 Not Available|[commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievement connection does work.|
|[Alien Breed: Impact](https://www.gog.com/game/alien_breed_impact)|126 (5/30/2022)|🔲 Not Available|[commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievement connection does work. Did not get to boot the game on Linux properly yet, however.|
|[Crypt of the NecroDancer](https://www.gog.com/game/crypt_of_the_necrodancer)|4.1.0-b5142 (4/3/2024)|🟩 Yes|[commit `ed38c3d`](https://github.com/kevin-wijnen/comet/commit/ed38c3d5253893779ba3d7ab828af442652f6044)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements do work. Leaderboard support works as of Comet version `ed38c3d`. Tested with game + all DLCs.|
|[Cuphead](https://www.gog.com/game/cuphead)|1.3.4 (8/19/2022)|🔲 Not Available|[commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122)|🟩 Yes|🟩 Achievements 🔲 Leaderboard|GalaxyCommunication.exe service required for game to start communicating with GOG. Otherwise, Achievements won't work. No Leaderboards present in-game.|
|[Cyberpunk 2077](https://www.gog.com/en/game/cyberpunk_2077)|2.21 (1/19/2025)|🔲 Not Available|[`v0.2.0`](https://github.com/imLinguin/comet/releases/tag/v0.2.0)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievements work in [Heroic v2.16.1](https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/releases/tag/v2.16.1). This was tested with the regular version of the game, not the [Ultimate Edition](https://www.gog.com/en/game/cyberpunk_2077_ultimate_edition) or [Phantom Liberty DLC](https://www.gog.com/en/game/cyberpunk_2077_phantom_liberty).|
|[DOOM + DOOM II](https://www.gog.com/game/doom_doom_ii)|Version 2265 - 8/7/2024|🔲 Not Available|[`v0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievements & Multiplayer do work as of `v0.1.2`.|
|[DOOM 3: BFG Edition](https://www.gog.com/game/doom_3)|1.14 (7/14/2017)|🔲 Not Available|[version `0.1.1`](https://github.com/imLinguin/comet/releases/tag/v0.1.1)|🟩 Yes|🟩 Achievements 🔲 Leaderboard|GalaxyCommunication.exe service required for game to start communicating with GOG. Otherwise, Achievements won't work. Mutliplayer isn't supported in GOG version of the game.|
|[DOOM (2016)](https://www.gog.com/game/doom_2016)|Version 20240321-110145-gentle-wolf - 2/25/2025|🔲 Not Available|[`v0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievements does work as of `v0.1.2`.|
|[Duck Detective: The Secret Salami](https://www.gog.com/game/duck_detective_the_secret_salami)|1.1.0|❌ Not Working|[version `v0.2.0`](https://github.com/imLinguin/comet/releases/tag/v0.2.0)|🔲 No|🟩 Achievements 🔲 Leaderboard|Windows build of the game is required to unlock achievements when on Linux.|
|[Ghostrunner](https://www.gog.com/game/ghostrunner)|42507_446 (6/24/2022)|🔲 Not Available|[version `0.1.0`](https://github.com/imLinguin/comet/releases/tag/v0.1.0)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements and Leaderboards work as expected. The game seems to separate saves based on Galaxy user id. Saves may need to be moved manually to be available.|
|[Horizon Zero Dawn Complete Edition](https://gog.com/game/horizon_zero_dawn_complete_edition)|7517962 (1/18/2022)|🔲 Not Available|[commit `c4715bf`](https://github.com/imLinguin/comet/commit/c4715bfa186f9b8955b842d57fd6f17fc5209f26)|🔲 No|🟩 Achievements 🔲 Leaderboard| Achievements do work.|
|[Indivisible](https://www.gog.com/game/indivisible)|42940 (6/22/2020)|🟩 Yes|[version 0.1.0](https://github.com/imLinguin/comet/releases/tag/v0.1.0)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievements do work.|
|[Kingdom Come: Deliverance](https://www.gog.com/game/kingdom_come_deliverance)|1.9.6-404-504czi3 |🔲 Not available|[version `0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🟩 Yes|🟩 Achievements 🔲 Leaderboard||
|[Metal Slug](https://www.gog.com/game/metal_slug)|Version gog-3 - 26/05/2017|🔲 Not Available|[version `v0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements and leaderboard do work as of `v0.1.2`. Multiplayer not tested yet.|
|[Metal Slug 2](https://www.gog.com/game/metal_slug_2)|gog-3 - 26/05/2017|🔲 Not Available|[version `0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements and leaderboard do work as of `v0.1.2`.|
|[Metal Slug 3](https://www.gog.com/game/metal_slug_3)|gog-5 - 26/05/2017|🔲 Not Available|[version `0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements and leaderboard do work as of `v0.1.2`. Multiplayer not tested yet.|
|[Metal Slug X](https://www.gog.com/game/metal_slug_x)|gog-6 - 02/06/2017|🔲 Not Available|[version `0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🟩 Leaderboard|Achievements and leaderboard do work as of `v0.1.2`. Multiplayer not tested yet.|
|[Quake II](https://www.gog.com/game/quake_ii_quad_damage)|5984 (11/01/2023)|🔲 Not available|[version `0.1.1`](https://github.com/imLinguin/comet/releases/tag/v0.1.1)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievements work as expected. The game needs OpenID support introduced in comet v0.1.1|
|[STONKS-9800: Stock Market Simulator](https://www.gog.com/game/stonks9800_stock_market_simulator)|0.4.2.5 (04/04/2024)|🔲 Not Available|[commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122)|❓ Unknown|❌ Achievements 🔲 Leaderboard|Game specific issue related to the GOG SDK library used. See [#26](https://github.com/imLinguin/comet/issues/26#issuecomment-2053667485) for any information and updates. Game did not connect to GOG via Comet with and without the dummy Service.|
|[Stardew Valley](https://www.gog.com/game/stardew_valley)|1.6.5.24110.6670590629 (4/19/2024)|🟩 Yes|[commit `c4715bf`](https://github.com/imLinguin/comet/commit/c4715bfa186f9b8955b842d57fd6f17fc5209f26)|🔲 No|🔲 Achievements 🔲 Leaderboard|The game uses Galaxy SDK for multiplayer only.|
|[Tomb Raider GOTY](https://www.gog.com/game/tomb_raider_goty)|1.0|🔲 Not available|[version `0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🔲 Leaderboard||
|[Wolfenstein: The New Order](https://www.gog.com/game/wolfenstein_the_new_order)|1.0.0.2 - 06/02/2020|🔲 Not Available|[version `0.1.2`](https://github.com/imLinguin/comet/releases/tag/v0.1.2)|🔲 No|🟩 Achievements 🔲 Leaderboard|Switch to older version without hotfix for the achievements, however the game is prone to crash. Refer to [this issue](https://github.com/imLinguin/comet/issues/57).|
|[Xeno Crisis](https://www.gog.com/game/xeno_crisis)|1.0.4 (2/11/2020)|❌ Not Working|[commit `55e4025`](https://github.com/imLinguin/comet/commit/55e402538df3bff354bf2e1e9a54fa4e5e091122)|🔲 No|🟩 Achievements 🔲 Leaderboard|Achievement connection does work. The GOG Galaxy communications are not present in the Linux version, thus the macOS or Windows version needs to be used with Comet to work.|
