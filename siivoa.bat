@echo off
powershell -Command "Remove-Item -Recurse -Force target\debug\incremental, target\mitta, target\tmp -ErrorAction SilentlyContinue; cargo clean --profile dev"
