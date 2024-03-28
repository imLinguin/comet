# GalaxyCommunication.exe

A dummy Windows service for Galaxy64.dll

This is the service that gets woken up by game process when Galaxy is not running already.

## Usage
This is only to be used within wine environment or on Windows (????) Where GOG Galaxy isn't installed.

In order to leverage this
1. Download or build GalaxyCommunication.exe
2. Register the service using the following command

```shell
sc create GalaxyCommunication binpath=<ABSOLUTE_COMMUNICATION_PATH>
```
`<ABSOLUTE_COMMUNICATION_PATH>` is an absolute path of downloaded GalaxyCommunication.exe like `C:\\ProgramData\\GOG.com\\Galaxy\\redists\\GalaxyCommunication.exe`

In case of Wine/Proton make sure to run the command above in the context of your prefix.

> [!TIP]  
> For Wine/Proton you can place the GalaxyCommunication.exe in C:\\windows\\system32, then binpath may be set to executable name

## Building

Use 
```shell
gcc -o GalaxyCommunication.exe main.c -ladvapi32
```

For cross compilation on Linux use `x86_64-w64-mingw32-gcc`