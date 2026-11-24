@echo off
setlocal
cd /d "%~dp0"
set "NoDefaultCurrentDirectoryInExePath="
for /f %%e in ('echo prompt $E^| cmd /d /q') do set "ESC=%%e"
set "VERDE=%ESC%[92m"
set "VERMELHO=%ESC%[91m"
set "AMARELO=%ESC%[93m"
set "NORMAL=%ESC%[0m"
set "CARGO_TOML=%~dp0native\Cargo.toml"
set "FIXTURES=%~dp0tests\fixtures.tsv"
set "PASSOS=%~dp0tests\passos.tsv"
set "POTENCIAS=%~dp0tests\potencias-corpus.tsv"
set "FIXTURES_CSHARP=%~dp0tests\fixtures-csharp.tsv"
set "AUDITORIA_CSHARP=%~dp0tests\RobustnessTests.exe"
set "ESPERAR=1"
if /i "%~1"=="--no-pause" set "ESPERAR="
call :limpar

echo.
echo [1/4] Rust: nucleo, auditoria independente e interface nativa...
where cargo >nul 2>nul
if errorlevel 1 goto semrust
set "SL_FIXTURES_SAIDA=%FIXTURES%"
set "SL_PASSOS_SAIDA=%PASSOS%"
set "SL_POTENCIAS_SAIDA=%POTENCIAS%"
cargo test --release --no-fail-fast --manifest-path "%CARGO_TOML%" -- --nocapture
if errorlevel 1 goto falharust
set "SL_FIXTURES_SAIDA="
set "SL_PASSOS_SAIDA="
set "SL_POTENCIAS_SAIDA="
if not exist "%FIXTURES%" goto semfixtures
if not exist "%PASSOS%" goto sempassos
if not exist "%POTENCIAS%" goto sempotencias

echo.
echo [2/4] Paridade com a versao C# de referencia (legado\csharp)...
set "CSC=%WINDIR%\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if not exist "%CSC%" set "CSC=%WINDIR%\Microsoft.NET\Framework\v4.0.30319\csc.exe"
if not exist "%CSC%" goto semcsharp
"%CSC%" /nologo /target:exe /out:"%AUDITORIA_CSHARP%" /reference:System.Numerics.dll "%~dp0legado\csharp\src\Solver.cs" "%~dp0legado\csharp\tests\RobustnessTests.cs"
if errorlevel 1 goto falhacsharp
"%AUDITORIA_CSHARP%" "%FIXTURES_CSHARP%"
if errorlevel 1 goto falhacsharp
fc /b "%FIXTURES%" "%FIXTURES_CSHARP%" >nul
if errorlevel 1 goto divergencia
echo %VERDE% OK %NORMAL% Rust e C# geraram as mesmas 76.674 respostas, byte a byte.
goto javascript
:semcsharp
echo %AMARELO% AVISO %NORMAL% Compilador do .NET Framework nao encontrado: paridade com C# nao verificada.

:javascript
echo.
echo [3/4] Versao HTML (JavaScript): respostas, potencias, modelo da tela e Modo pensar...
where node >nul 2>nul
if errorlevel 1 goto semnode
node "%~dp0tools\build-portable.cjs"
if errorlevel 1 goto falhajs
node "%~dp0tests\PortableTests.cjs" "%FIXTURES%" "%PASSOS%" "%POTENCIAS%"
if errorlevel 1 goto falhajs

echo.
echo [4/4] Interface HTML no Google Chrome, sem janela...
node "%~dp0tests\InterfaceHtml.cjs"
if errorlevel 3 goto semchrome
if errorlevel 1 goto falhainterface
goto concluido
:semchrome
if defined CI goto faltachrome
echo %AMARELO% AVISO %NORMAL% Google Chrome nao encontrado: interface HTML nao testada.
goto concluido
:semnode
echo %AMARELO% AVISO %NORMAL% Node.js nao encontrado: versao HTML nao reconstruida nem testada.

:concluido
call :limpar
echo.
echo %VERDE% OK %NORMAL% Validacao concluida.
if defined ESPERAR pause
exit /b 0

:semrust
set "MSG=Rust nao encontrado. Instale em https://rustup.rs e rode de novo."
goto falha
:falharust
set "MSG=Os testes do Rust falharam (veja acima)."
goto falha
:semfixtures
set "MSG=A auditoria do Rust nao gerou o arquivo de respostas."
goto falha
:sempassos
set "MSG=Os testes do Rust nao geraram o arquivo de passos do Modo pensar (tests\passos.tsv)."
goto falha
:sempotencias
set "MSG=Os testes do Rust nao geraram o corpus de potencias (tests\potencias-corpus.tsv)."
goto falha
:falhacsharp
set "MSG=Nao foi possivel rodar a auditoria da versao C# de referencia."
goto falha
:divergencia
set "MSG=Rust e C# divergiram em alguma resposta. Compare tests\fixtures.tsv com tests\fixtures-csharp.tsv."
echo.
echo %VERMELHO% ERRO %NORMAL% %MSG%
if defined ESPERAR pause
exit /b 1
:falhajs
set "MSG=A versao HTML falhou na validacao (veja acima)."
goto falha
:falhainterface
set "MSG=A interface HTML falhou no Google Chrome (veja acima)."
goto falha
:faltachrome
set "MSG=Google Chrome nao encontrado: na integracao continua a interface HTML precisa ser testada."
goto falha

:falha
call :limpar
echo.
echo %VERMELHO% ERRO %NORMAL% %MSG%
if defined ESPERAR pause
exit /b 1

:limpar
if exist "%FIXTURES%" del "%FIXTURES%"
if exist "%PASSOS%" del "%PASSOS%"
if exist "%POTENCIAS%" del "%POTENCIAS%"
if exist "%FIXTURES_CSHARP%" del "%FIXTURES_CSHARP%"
if exist "%AUDITORIA_CSHARP%" del "%AUDITORIA_CSHARP%"
exit /b 0
