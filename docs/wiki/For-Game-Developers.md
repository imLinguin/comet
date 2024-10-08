> [!IMPORTANT]
> This is **not** an official page affiliated with GOG. These are my findings while I was looking into the SDK client libraries that GOG provides.  
> While the libraries disclosed below have been proven to work in production, **there is no guarantee.**  
> Nevertheless I encourage you to at least try linking your game against them and see the results.

## You made it here!

If you're seeing this page, then you're probably considering shipping a native Linux build of your game on GOG.  
Excellent choice! If you've researched the GOG GALAXY SDK then you've probably discovered that it appears to not be available for Linux.

**But don't leave yet!** It is still possible to use the GOG GALAXY SDK on your Linux build to enable achievements and other online features, and it may even save you some general effort in the process.  
This has already been proven to work by several games, including:

- [Crypt of the NecroDancer](https://www.gog.com/game/crypt_of_the_necrodancer)
- [Indivisible](https://www.gog.com/game/indivisible)
- [Stardew Valley](https://www.gog.com/game/stardew_valley)
- [Streets of Rage 4](https://www.gog.com/game/streets_of_rage_4)

## How to get the native Linux library

In the Developer Portal in `Galaxy Components` > `SDK`, you'll see the following download section:

![image](https://github.com/user-attachments/assets/3bdc9728-ad09-4bd4-83c4-09c745433a8a)

The `Steam-runtime` is what you are looking for. Pick an appropriate bitness and link your app against that.
This build is up-to date like the rest of the SDK packages.

> [!NOTE]
> There is no GalaxyPeer library for Linux. Instead, it seems to be statically linked within the provided libGalaxy.so library.

## Why bother?

Going out of your way to use the library that GOG is curiously neglecting to mention may feel weird, but there are significant advantages.

1. DLC Discovery via GOG GALAXY SDK: https://docs.gog.com/sdk-dlc-discovery
   - GOG normally encourages you to build your own solution for DLC Discovery on your Linux build, but now you can continue using the dedicated GOG GALAXY SDK method, just as you would on your Windows build.

2. Potentially less work in general
   - To ease the burden in supporting multiple storefronts, this library may generally reduce the amount of platform-specific workarounds needed to offset the inconsistent feature sets between platforms on GOG.

3. Multiplayer, Leaderboards & more on Linux
   - Just like your other builds, your Linux build can now provide multiplayer functionality and player authentication features. While the GOG GALAXY client is not available for Linux, players can use an equivalent client such as `comet` to pick up your game's API calls.\
   *(Remember to keep these online features optional in order for your game to remain DRM-free - see the [DRM and SDK](https://docs.gog.com/sdk/#drm-and-sdk) section of the GOG Dev Docs for more information.)*

## So what is `comet`?

Comet is a drop-in replacement for the GOG GALAXY client that allows players to leverage "GOG GALAXY-only" functionality (multiplayer, achievements, stats and leaderboards) on every platform that GOG games are available for.

The main goal of this project is to bring the GOG GALAXY feature set to Linux users and let them use the platform to the fullest.

Comet has already shipped to thousands of users through the [Heroic Games Launcher](https://heroicgameslauncher.com/), with more Linux clients soon to follow.

## Not convinced?

Depending on the features that your game provides, not using this library may be more troublesome than expected; you should evaluate this per your project scope.

Should you ultimately decide against including the `Steam-runtime` library, your choice will be respected. Linux users who wish to access your game's online features can still use Proton to run the Windows build of your game, so not everything is lost.

*Have a nice day!*  
*from the Comet developers*
