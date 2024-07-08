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

echo .
:: Check if nodejs is presnet
echo Checking if nodejs is installed...
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo Node.js not installed. Would you like to use fnm?
    choice /C YN /M "Automatically install Node.js? [Y/N]"
    if errorlevel 2 (
        echo User chose to use fnm. Please install Node.js from https://nodejs.org/en/download/ and try again.
        EXIT /B 1
    ) else (
        echo User chose to use fnm. Proceeding with fnm installation...
        
        :: Install fnm using winget
        call winget install Schniz.fnm
        call %vendorPath%\refreshenv.bat
        
        :: Use fnm to install Node.js version 20
        call fnm use --install-if-missing 20
        
        echo Done
        call node -v
        call npm -v
    )
) else (
    echo Node.js is already installed.
)
echo .

echo Installing project dependencies...

npm install

echo .
echo Building project...

npm run build

echo .

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