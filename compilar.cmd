@echo off
setlocal
cd /d "%~dp0"
set "CSC=%WINDIR%\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if not exist "%CSC%" set "CSC=%WINDIR%\Microsoft.NET\Framework\v4.0.30319\csc.exe"
if not exist "%CSC%" goto missing
"%CSC%" @compilacao.rsp
if errorlevel 1 goto failed
"%CSC%" /nologo /target:exe /out:tests\SolverTests.exe /reference:System.Numerics.dll src\Solver.cs tests\SolverTests.cs
if errorlevel 1 goto failed
tests\SolverTests.exe
if errorlevel 1 goto failed
"%CSC%" /nologo /target:exe /out:InterfaceTests.exe /reference:SistemasLineares.exe /reference:System.Windows.Forms.dll /reference:System.Drawing.dll tests\InterfaceTests.cs
if errorlevel 1 goto failed
InterfaceTests.exe
if errorlevel 1 goto failed
del tests\SolverTests.exe
del InterfaceTests.exe
echo Aplicativo gerado: %CD%\SistemasLineares.exe
if /i not "%~1"=="--no-pause" pause
exit /b 0
:missing
echo Compilador do .NET Framework 4 nao encontrado.
goto failed
:failed
echo Nao foi possivel concluir. Feche o aplicativo antes de recompilar.
if /i not "%~1"=="--no-pause" pause
exit /b 1
