# GalaxyCommunication.exe

A dummy Windows service for Galaxy64.dll

This is the service that gets woken up by game process when Galaxy is not running already.

## Usage
> [!TIP]
> For easy installation, have `GalaxyCommunication.exe` and `install-dummy-service.bat` both in a folder Heroic has access to and run the script.
>
> For Heroic Games Launcher users on Linux and Mac: run the `.bat` file by dragging it into the `RUN EXE ON PREFIX` box of the game's WINE settings (in Heroic).

This is only to be used within wine environment or on Windows (????), where GOG Galaxy isn't installed.

In order to leverage this
1. Download or build GalaxyCommunication.exe
2. Register the service using the following command or `install-dummy-service.bat` script

> [!NOTE]
> On Windows you may need admin privileges

```shell
sc create GalaxyCommunication binpath=<ABSOLUTE_COMMUNICATION_PATH>
```
`<ABSOLUTE_COMMUNICATION_PATH>` is an absolute path of downloaded GalaxyCommunication.exe like `C:\\ProgramData\\GOG.com\\Galaxy\\redists\\GalaxyCommunication.exe`

In case of Wine/Proton make sure to run the command above in the context of your prefix.

## Building

Use 
```shell
gcc -o GalaxyCommunication.exe main.c -ladvapi32
```

For cross compilation on Linux use `x86_64-w64-mingw32-gcc`