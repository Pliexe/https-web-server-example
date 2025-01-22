@echo off

echo.
echo Starting building of project
echo.
echo Building for Windows
echo.

call cargo build --release

echo.
echo Building for Linux
echo.

call wsl ./build.sh

echo.
echo Done.
echo.

if exist build rmdir /s /q build

mkdir build
mkdir build\windows
mkdir build\windows\vendor
mkdir build\linux

echo.
echo Packaging...
echo.

copy "target\release\https-web-server-example.exe" "build\windows\https-web-server-example.exe"
copy "target\release\https-web-server-example" "build\linux\https-web-server-example"

copy "target\release\https-web-server-example.exe" "build\https-web-server-example-win64.exe"
copy "target\release\https-web-server-example" "build\https-web-server-example-linux64"

xcopy "public\*" "build\windows\public\" /s /e
xcopy "public\*" "build\linux\public\" /s /e

copy "generate_certs.bat" "build\windows\generate_certs.bat"
copy "generate_certs.sh" "build\linux\generate_certs.sh"

copy "LICENSE" "build\windows\LICENSE"
copy "LICENSE" "build\linux\LICENSE"

copy "README.md" "build\windows\README.md"
copy "README.md" "build\linux\README.md"


copy "vendor\refreshenv.bat" "build\windows\vendor\refreshenv.bat"

cd build/windows
7z a ../win64.7z *
cd ../..
cd build/linux
7z a ../linux64.7z *
cd ../.. 
@REM In case something is added in future ^

echo.
echo Done.
echo.

pause