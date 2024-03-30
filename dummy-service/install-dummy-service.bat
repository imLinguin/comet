@echo off

SET currdir=%~dp0
SET targetdir=C:\ProgramData\GOG.com\Galaxy\redists

sc create GalaxyCommunication binpath=%targetdir%\GalaxyCommunication.exe
if not exist "%targetdir%" mkdir %targetdir% 
xcopy /y /q %currdir%GalaxyCommunication.exe %targetdir%
