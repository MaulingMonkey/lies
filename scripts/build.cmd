@set ERRORS=0
@pushd "%~dp0.."
@call :cargo +nightly rustc -p example-console
@call :cargo test
@call :cargo doc --no-deps
@if NOT "%ERRORS%" == "0" exit /b %ERRORS%
target\debug\example-console version
target\debug\example-console help
target\debug\example-console about
target\debug\example-console add 1 2 3
@popd
@if "%ERRORS%" == "0" @call "%~dp0publish.cmd" --dry-run
@exit /b %ERRORS%

:cargo
cargo %*
@if ERRORLEVEL 1 set /A ERRORS=%ERRORS%+1
@exit /b %ERRORLEVEL%
