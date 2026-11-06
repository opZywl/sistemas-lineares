@echo off
setlocal
cd /d "%~dp0"
call compilar.cmd --no-pause
if errorlevel 1 goto failed
set "CSC=%WINDIR%\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if not exist "%CSC%" set "CSC=%WINDIR%\Microsoft.NET\Framework\v4.0.30319\csc.exe"
"%CSC%" /nologo /target:exe /out:RobustnessTests.exe /reference:System.Numerics.dll src\Solver.cs tests\RobustnessTests.cs
if errorlevel 1 goto failed
RobustnessTests.exe tests\fixtures.tsv
if errorlevel 1 goto failed
where node >nul 2>nul
if errorlevel 1 goto skipportable
node tools\build-portable.cjs
if errorlevel 1 goto failed
node tests\PortableTests.cjs tests\fixtures.tsv
if errorlevel 1 goto failed
:skipportable
del RobustnessTests.exe
del tests\fixtures.tsv
echo Validacao concluida.
if /i not "%~1"=="--no-pause" pause
exit /b 0
:failed
echo A validacao encontrou um erro. Confira as mensagens acima.
if /i not "%~1"=="--no-pause" pause
exit /b 1
