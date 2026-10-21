# MdReader

눈이 편한 화이트 톤의 Markdown 리더/에디터 (Windows, Tauri).

- 폴더 탐색기 · 여러 탭 · 우측 목차 · 보기/편집/분할 모드 · `Ctrl+S` 저장
- 터미널에서 VS Code처럼 `mdr .` 로 폴더 열기
- GitHub Releases 기반 자동 업데이트

## 설치

[Releases](https://github.com/pyjhoop/MdReader/releases/latest)에서 `MdReader_*_x64-setup.exe`를 받아 설치합니다.
설치 후 앱을 **한 번 실행**하면 `mdr` 명령이 PATH에 등록됩니다. (새 터미널부터 적용)

```powershell
mdr .              # 현재 폴더 열기
mdr C:\docs        # 특정 폴더
mdr README.md      # 파일 하나를 탭으로
mdr                # 마지막 폴더와 열어둔 탭 복원
```

이미 실행 중이면 새 창을 띄우지 않고 기존 창에서 열립니다. 이후 새 버전은 앱이 실행될 때 스스로 확인해 업데이트합니다.

## 단축키

| 키 | 동작 |
| --- | --- |
| `Ctrl+S` | 저장 |
| `Ctrl+E` | 보기 ↔ 분할 |
| `Alt+N` / `Alt+W` | 새 문서 / 탭 닫기 |
| `B` / `T` | 탐색기 / 목차 토글 |
| `/` | 파일 검색 |
| `+` / `-` | 글자 크기 |

## 개발

필요: Node.js, Rust, WebView2

```powershell
npm install
npm run dev        # ui/ 를 고치면 창이 자동 새로고침, Rust를 고치면 자동 재빌드
npm run build      # 설치 파일 빌드 (서명 키 환경변수 필요, 아래 참고)
```

프런트엔드는 번들러 없이 `ui/index.html` 한 파일이고, 라이브러리는 `ui/vendor/`에 포함되어 있어 오프라인에서도 동작합니다.

## 릴리스 (자동 업데이트 배포)

1. 한 번만: 서명 키 생성 → 공개키를 `src-tauri/tauri.conf.json`의 `plugins.updater.pubkey`에, 비밀키를 GitHub 저장소 Secrets에 등록
   ```powershell
   npx tauri signer generate -w $env:USERPROFILE\.tauri\mdreader.key
   ```
   - `TAURI_SIGNING_PRIVATE_KEY` : 키 파일(`mdreader.key`) 내용
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` : 키 비밀번호 (없으면 빈 값)
2. 새 버전 배포
   ```powershell
   .\scripts\release.ps1 0.1.1
   ```
   버전 갱신 → 커밋 → `v0.1.1` 태그 push → GitHub Actions가 빌드해 Releases에 업로드합니다. 설치된 앱은 다음 실행 때 업데이트를 안내합니다.

> 비밀키(`mdreader.key`)는 절대 저장소에 커밋하지 마세요. 잃어버리면 기존 사용자에게 업데이트를 배포할 수 없습니다.
