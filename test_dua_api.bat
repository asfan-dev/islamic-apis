@echo off
setlocal enabledelayedexpansion

REM ========================================
REM     SIMPLE DUA API TEST SUITE
REM ========================================

REM Configuration
set SERVER=localhost
set PORT=3000
set BASE=http://%SERVER%:%PORT%

REM Counters
set /a PASS=0
set /a FAIL=0
set /a TOTAL=0

cls
echo =========================================
echo      SIMPLE DUA API TEST SUITE
echo =========================================
echo.
echo Server: %BASE%
echo.

REM Test Health Check
echo Testing: Health Check
curl -s -o nul -w "%%{http_code}" "%BASE%/health" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Basic Endpoints
echo.
echo Testing: List All Duas
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: List Duas with Pagination
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?page=1&per_page=5" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Random Dua
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas/random" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else if "%CODE%"=="404" (
    echo   [WARN] Status: %CODE% - May be empty
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Filters
echo.
echo Testing: Filter by Morning
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?invocation_time=morning" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Filter by Source Type
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?source_type=Quran" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Search by Keyword
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?q=bismillah" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Sort by Popularity
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?sort=popularity_score&order=desc" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Categories
echo.
echo Testing: List Categories
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/categories" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Get Category
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/categories/daily-life" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Category Duas
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/categories/daily-life/duas" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Tags
echo.
echo Testing: List Tags
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/tags" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Tag Duas
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/tags/essential/duas" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Bundles
echo.
echo Testing: List Bundles
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/bundles" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Get Bundle
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/bundles/morning-adhkar" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Bundle Items
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/bundles/morning-adhkar/items" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Sources
echo.
echo Testing: List Sources
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/sources" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Media
echo.
echo Testing: List Media
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/media" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Search
echo.
echo Testing: Keyword Search
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/search?q=protection" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Suggestions
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/suggest?q=mor" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Stats
echo.
echo Testing: Statistics
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/stats" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Translations
echo.
echo Testing: All Translations
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/translations" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Test Complex Queries
echo.
echo Testing: Complex Query 1
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?invocation_time=morning&authenticity=Sahih" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Complex Query 2
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?source_type=Quran&popularity_min=0.5" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

echo Testing: Include Relations
curl -s -o nul -w "%%{http_code}" "%BASE%/v1/duas?include=sources,media,context" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM POST Request Test
echo.
echo Testing: Semantic Search (POST)
curl -s -o nul -w "%%{http_code}" -X POST "%BASE%/v1/search/semantic" -H "Content-Type: application/json" -d "{\"query\":\"anxiety\",\"limit\":5}" > status.txt
set /p CODE=<status.txt
set /a TOTAL+=1
if "%CODE%"=="200" (
    echo   [PASS] Status: %CODE%
    set /a PASS+=1
) else (
    echo   [FAIL] Status: %CODE%
    set /a FAIL+=1
)
del status.txt

REM Summary
echo.
echo =========================================
echo              TEST SUMMARY
echo =========================================
echo.
echo Passed: %PASS%
echo Failed: %FAIL%
echo Total:  %TOTAL%
echo.

set /a RATE=(%PASS%*100)/%TOTAL%
echo Success Rate: %RATE%%%
echo.

pause