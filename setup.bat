@echo off
setlocal enabledelayedexpansion

@echo Begin setup...

:: Get the directory of the current batch file
set "batchDir=%~dp0"

:: Construct the path to the "vendor" folder
set "vendorPath=%batchDir%vendor"

:: Check if the "vendor" folder exists; if not, create it
if not exist "%vendorPath%" (
    echo Folder "vendor" missing. Creating folder.
    mkdir "%vendorPath%" || EXIT /B 1
)

:: Set the URL of the file to download
set "url=https://dl.filippo.io/mkcert/v1.4.4?for=windows/amd64"

:: Specify the filename
set "filename=mkcert_windows_amd64.exe"

:: Construct the full path to the file in the "vendor" folder
set "fullPath=%vendorPath%\%filename%"

:: Check if the file already exists; if not, download it
if not exist "%fullPath%" (
    @echo Missing file "mkcert_windows_amd64.exe". Downloading from url...
    curl -L -o "%fullPath%" "%url%" || EXIT /B 1
)

:: Create the "certs" folder if it doesn't exist
if not exist "%batchDir%certs" (
    echo Folder "certs" missing. Creating folder.
    mkdir "%batchDir%certs" || EXIT /B 1
)

:: Change the working directory to the "certs" folder
echo Changing working directory to "certs".
cd "%batchDir%certs" || EXIT /B 1

:: Call mkcert to install the local CA
echo Installing local CA...
call "%fullPath%" -install || EXIT /B 1

:: Generate certificates for the specified domains
echo Generating certificates for localhost, 127.0.0.1, and ::1...
call "%fullPath%" --key-file localhost-key.pem --cert-file localhost.pem localhost 127.0.0.1 ::1 || EXIT /B 1

echo Setup completed successfully.

endlocal

:: Check if the batch file was run from a command window
echo %cmdcmdline:"=-% | find /i "cmd /c --%~dpf0%-"
if %errorlevel% NEQ 0 (
    :: Batch file was executed from Windows Explorer, add a pause to keep the console open
    pause
) else (
    :: Batch file was executed from within a Command Prompt, do not add a pause
    rem Your commands here
)