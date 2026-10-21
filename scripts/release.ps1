# 새 버전을 릴리스한다.   사용: .\scripts\release.ps1 0.1.1
#  1) tauri.conf.json / package.json / Cargo.toml 버전 갱신  2) 커밋  3) v<버전> 태그 생성  4) push
#  push 된 태그를 GitHub Actions 가 받아 빌드 → Releases 업로드 → 설치된 앱이 자동 업데이트
param([Parameter(Mandatory)][string]$Version)
$ErrorActionPreference = 'Stop'
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "버전 형식은 0.1.1 처럼 입력하세요." }
Set-Location (Split-Path $PSScriptRoot)
if (git status --porcelain) { throw "커밋되지 않은 변경이 있습니다. 먼저 커밋하세요." }

function Set-Version($file, $pattern, $replacement) {
    $text = Get-Content $file -Raw
    $new = [regex]::Replace($text, $pattern, $replacement, 'Multiline')
    if ($new -eq $text) { throw "$file 에서 버전을 찾지 못했습니다." }
    [IO.File]::WriteAllText((Resolve-Path $file), $new, (New-Object Text.UTF8Encoding $false))
}
Set-Version 'src-tauri\tauri.conf.json' '("version":\s*")[^"]+(")' "`${1}$Version`${2}"
Set-Version 'package.json'              '("version":\s*")[^"]+(")' "`${1}$Version`${2}"
Set-Version 'src-tauri\Cargo.toml'      '^(version\s*=\s*")[^"]+(")' "`${1}$Version`${2}"

git add -A
git commit -m "release v$Version"
git tag "v$Version"
git push origin HEAD "v$Version"
Write-Host "v$Version 을 push 했습니다. GitHub Actions 탭에서 빌드 진행을 확인하세요."
