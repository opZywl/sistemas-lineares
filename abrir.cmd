@echo off
setlocal
cd /d "%~dp0"
set "NoDefaultCurrentDirectoryInExePath="
title Sistemas Lineares - abrir
for /f %%e in ('echo prompt $E^| cmd /d /q') do set "ESC=%%e"
set "VERDE=%ESC%[92m"
set "VERMELHO=%ESC%[91m"
set "AMARELO=%ESC%[93m"
set "NORMAL=%ESC%[0m"
set "APP=%~dp0SistemasLineares.exe"
set "HTML=%~dp0SistemasLineares.html"
set "CARGO_TOML=%~dp0native\Cargo.toml"
set "COMPILADO=%~dp0native\target\release\SistemasLineares.exe"
set "MODO=app"
set "ESPERAR=1"

:argumentos
if "%~1"=="" goto inicio
if /i "%~1"=="html" set "MODO=html"
if /i "%~1"=="instalar" set "MODO=instalar"
if /i "%~1"=="desinstalar" set "MODO=desinstalar"
if /i "%~1"=="--no-pause" set "ESPERAR="
shift
goto argumentos

:inicio
echo.
echo  Sistemas Lineares - Lucas Lima - RA 185964
echo.
if "%MODO%"=="html" goto html
if "%MODO%"=="desinstalar" goto desinstalar

echo [1/5] Verificando o Rust...
where cargo >nul 2>nul
if errorlevel 1 goto semrust

echo [2/5] Compilando a versao mais recente (na primeira vez demora alguns minutos)...
cargo build --release --manifest-path "%CARGO_TOML%"
if errorlevel 1 goto falhacompilacao

echo [3/5] Rodando os testes...
cargo test --release --quiet --manifest-path "%CARGO_TOML%"
if errorlevel 1 goto falhatestes

echo [4/5] Atualizando o SistemasLineares.exe...
call :fecharaberto
if errorlevel 1 goto aindaaberto
copy /y "%COMPILADO%" "%APP%" >nul
if errorlevel 1 goto falhacopia
if "%MODO%"=="instalar" goto instalar

:abrir
echo [5/5] Abrindo o aplicativo...
start "" "%APP%"
set /a TENTATIVAS=0
:aguardarjanela
call :janelaaberta
if not errorlevel 1 goto aberto
set /a TENTATIVAS+=1
if %TENTATIVAS% geq 20 goto naoabriu
ping -n 2 127.0.0.1 >nul
goto aguardarjanela

:aberto
echo.
echo %VERDE% OK %NORMAL% Sistemas Lineares aberto: "%APP%"
echo      Para fechar, feche a janela do aplicativo.
echo      Versao HTML offline no Chrome: abrir.cmd html
goto fim

:semrust
if not exist "%APP%" goto semrustsemapp
echo %AMARELO% AVISO %NORMAL% Rust nao encontrado: usando o SistemasLineares.exe ja compilado, sem recompilar.
echo        Para recompilar, instale o Rust em https://rustup.rs
if "%MODO%"=="instalar" goto instalar
goto abrir

:instalar
echo [5/5] Instalando no Windows (menu Iniciar e Aplicativos instalados)...
start "" /wait "%APP%" --instalar
if errorlevel 1 goto falhainstalar
echo.
echo %VERDE% OK %NORMAL% Instalado em "%LOCALAPPDATA%\Programs\Sistemas Lineares".
echo      Abra pelo menu Iniciar: Sistemas Lineares.
echo      Para remover: Configuracoes, Aplicativos instalados, ou abrir.cmd desinstalar
goto fim

:desinstalar
set "INSTALADO=%LOCALAPPDATA%\Programs\Sistemas Lineares\SistemasLineares.exe"
set "DESINSTALADOR=%APP%"
if exist "%INSTALADO%" set "DESINSTALADOR=%INSTALADO%"
if not exist "%DESINSTALADOR%" goto naoinstalado
echo [1/1] Removendo a instalacao do Windows...
start "" /wait "%DESINSTALADOR%" --desinstalar
if errorlevel 2 goto desinstalacaocancelada
if errorlevel 1 goto falhadesinstalar
echo.
echo %VERDE% OK %NORMAL% Sistemas Lineares removido do menu Iniciar e de Aplicativos instalados.
goto fim
:desinstalacaocancelada
echo.
echo %AMARELO% AVISO %NORMAL% Desinstalacao cancelada.
goto fim

:html
echo [1/2] Atualizando a versao HTML...
where node >nul 2>nul
if errorlevel 1 goto htmlsemnode
node "%~dp0tools\build-portable.cjs"
if errorlevel 1 goto falhahtml
goto htmlabrir
:htmlsemnode
echo %AMARELO% AVISO %NORMAL% Node.js nao encontrado: usando o SistemasLineares.html ja gerado.
:htmlabrir
if not exist "%HTML%" goto semhtml
echo [2/2] Abrindo no Chrome...
call :chrome
if not defined CHROME goto semchrome
start "" "%CHROME%" "%HTML%"
echo.
echo %VERDE% OK %NORMAL% Versao HTML aberta no Chrome: "%HTML%"
goto fim

:falhacompilacao
set "MSG=A compilacao falhou (veja acima). Se faltar o link.exe, instale o Visual Studio Build Tools com C++."
goto falha
:falhatestes
set "MSG=Os testes falharam (veja acima). O SistemasLineares.exe da raiz nao foi alterado."
goto falha
:aindaaberto
set "MSG=O Sistemas Lineares continua aberto. Feche a janela dele e rode abrir.cmd de novo."
goto falha
:falhacopia
set "MSG=Nao foi possivel atualizar o SistemasLineares.exe da raiz."
goto falha
:naoabriu
set "MSG=O aplicativo nao abriu em 20 segundos."
goto falha
:semrustsemapp
set "MSG=Rust nao encontrado e nao ha SistemasLineares.exe. Instale o Rust em https://rustup.rs ou use: abrir.cmd html"
goto falha
:falhahtml
set "MSG=Nao foi possivel gerar o SistemasLineares.html."
goto falha
:semhtml
set "MSG=SistemasLineares.html nao encontrado."
goto falha
:semchrome
set "MSG=Google Chrome nao encontrado. Abra o SistemasLineares.html com dois cliques em um navegador atual."
goto falha
:falhainstalar
set "MSG=A instalacao nao foi concluida (veja a mensagem do aplicativo)."
goto falha
:falhadesinstalar
set "MSG=A desinstalacao nao foi concluida (veja a mensagem do aplicativo)."
goto falha
:naoinstalado
set "MSG=Nenhum SistemasLineares.exe encontrado para desinstalar."
goto falha

:falha
echo.
echo %VERMELHO% ERRO %NORMAL% %MSG%
if defined ESPERAR pause
exit /b 1

:fim
if defined ESPERAR ping -n 6 127.0.0.1 >nul
exit /b 0

:rodando
tasklist /fi "imagename eq SistemasLineares.exe" /nh 2>nul | findstr /i /c:"SistemasLineares.exe" >nul
exit /b %errorlevel%

:janelaaberta
tasklist /fi "imagename eq SistemasLineares.exe" /fi "windowtitle eq Sistemas Lineares*" /nh 2>nul | findstr /i /c:"SistemasLineares.exe" >nul
exit /b %errorlevel%

:fecharaberto
call :rodando
if errorlevel 1 exit /b 0
taskkill /im SistemasLineares.exe >nul 2>nul
set /a ESPERA=0
:fecharaberto_aguardar
call :rodando
if errorlevel 1 exit /b 0
set /a ESPERA+=1
if %ESPERA% geq 10 exit /b 1
ping -n 2 127.0.0.1 >nul
goto fecharaberto_aguardar

:chrome
set "CHROME="
if exist "%ProgramFiles%\Google\Chrome\Application\chrome.exe" set "CHROME=%ProgramFiles%\Google\Chrome\Application\chrome.exe"
if not defined CHROME if exist "%ProgramFiles(x86)%\Google\Chrome\Application\chrome.exe" set "CHROME=%ProgramFiles(x86)%\Google\Chrome\Application\chrome.exe"
if not defined CHROME if exist "%LocalAppData%\Google\Chrome\Application\chrome.exe" set "CHROME=%LocalAppData%\Google\Chrome\Application\chrome.exe"
exit /b 0
