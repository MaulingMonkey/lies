@set ERRORS=0
@pushd "%~dp0.."

@cd "%~dp0../crates/lies-impl"
@call :cargo publish %*
@ping localhost -n 10 >NUL 2>NUL
@cd "%~dp0../crates/lies
@call :cargo publish %*

@popd
@exit /b %ERRORS%

:cargo
cargo %*
@if ERRORLEVEL 1 set /A ERRORS=%ERRORS%+1
@exit /b %ERRORLEVEL%
