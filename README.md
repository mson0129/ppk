# 돌아온 너구리 · Rust/WASM 포트

게임 로직은 [Rust 엔진](rust/README.md)에, 브라우저 화면·입력·오디오·
리플레이·타이머·오프라인 지원 코드는 `web/`에 있습니다.

## 로컬 빌드

Python 3과 Rust의 `wasm32-unknown-unknown` 타깃이 필요합니다.
빌드에 필요한 게임 데이터 7개는 `assets/`에 포함되어 있습니다.
저장소를 받은 뒤 바로 빌드할 수 있습니다.

```sh
rustup target add wasm32-unknown-unknown
./build.sh
python3 -m http.server 8081 --bind 127.0.0.1 --directory dist
```

브라우저에서 `http://127.0.0.1:8081/`을 엽니다. 빌드는 `assets/`의 데이터를 WASM에
포함하고, 실행에 필요한 웹 파일을 `dist/`에 생성합니다. 원본 DOS 실행 파일,
ZIP 아카이브, 분석 자료는 빌드에 필요하지 않습니다.

## 실행과 검증

방향키로 이동·사다리 타기, Space로 시작·점프, P로 일시정지합니다.
터치 컨트롤과 게임패드도 지원합니다. 타임어택은 타이틀에서 Space를 누를
때 시작해 50번 스테이지의 마지막 사탕을 먹을 때 끝납니다.
리플레이 재생에는 녹화 시점과 같은 WASM 빌드가 필요합니다.

빌드 후 Node.js와 Python의 `wasmtime` 패키지로 회귀 테스트를 실행합니다.

```sh
node --test tests/*.test.mjs
python3 -m unittest discover -s tests -p test_wasm.py
python3 -m unittest discover -s tests -p test_pwa.py
```

## GitHub Pages 준비

나중에 이 저장소에서 Pages를 배포할 때는 빌드한 뒤
`dist/`를 정적 사이트 산출물로 사용합니다. 프로젝트 사이트 경로는
`https://mson0129.github.io/ppk/`이며, 매니페스트와 서비스 워커는
문서 위치에 상대적인 경로를 사용합니다. HTTPS에서 앱을 한 번 열고
오프라인 준비가 완료되면 홈 화면 앱에서 오프라인으로 플레이할 수 있습니다.
