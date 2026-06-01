@echo off
echo Starting Nexus Backend (FastAPI on port 8000)...
cd /d D:\nexus\backend
if exist venv\Scripts\activate.bat call venv\Scripts\activate.bat
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
pause
