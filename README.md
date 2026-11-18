# 돌아온 너구리 Rust WASM 포팅

[돌아온 너구리 Rust WASM 포팅 버전](https://mson0129.github.io/ppk/)

DOS 게임인 돌아온 너구리를 리버스 엔지니어링을 통해 웹 버전으로 포팅한 것입니다.\
게임 엔진을 Rust로 재작성하고 WebAssembly(WASM)로 빌드했습니다.

돌아온 너구리의 원 저작자는 아래와 같습니다.

Copyright (c) 1992 돌쇠 & 황성규

## 코드 설명

### 엔진

`src/`는 게임 규칙, 화면 렌더링, 소리와 장면 전환을 담당합니다.\
`web/`는 브라우저에서 화면을 표시하고 키보드·터치·게임패드 입력,
타임어택, 리플레이, 오프라인 실행을 처리합니다.

| 파일 |읽을 내용 |
| --- | --- |
| `state.rs` | `Player`, `Walker`, `Hatchling`, `FloatingScore`, `GameState` |
| `player.rs` | 방향 전환, 걷기, 사다리 정렬, 세 종류의 점프 궤적 |
| `enemies.rs` | 일반·고속 적, 부화한 적, 접촉 판정과 점수 표시 이동 |
| `items.rs` | 사탕 연속 점수, 알 부화, 낭떠러지와 가시 판정 |
| `stage.rs` | 원본 50개 맵의 문자 코드 → 충돌 격자·적 생성 |
| `engine.rs` | `Phase`에 따른 시작·플레이·사망·이어하기·클리어 전환 |
| `graphics.rs`, `render.rs` | 원본 비트플레인 이미지, 마스크, 배경·HUD·보간 렌더링 |
| `audio.rs` | `Tune`별 점프·수집·부화 소리와 음표 진행 |
| `transition.rs` | 70.0863Hz 기준 RGB 팔레트 페이드 |
| `compat.rs` | 기존 C 테스트·웹 ABI용 데이터 이미지 직렬화·역직렬화 |
| `lib.rs` | 브라우저 JavaScript가 호출하는 WASM 함수 |
| `assets.rs` | 게임 데이터 파일과 엔진에서 사용하는 에셋 위치 상수 |

`Engine::tick_game`에서 한 게임 틱의 처리 순서를 볼 수 있습니다. 게임 로직은
`game.player.x`, `game.remaining_candy`처럼 이름 있는 필드를 사용합니다.
`compat.rs`의 주소 대응은 원본과 바이트 단위로 비교하고 기존 디버그 인터페이스를
유지하기 위한 경계 계층입니다. CPU 레지스터·플래그·스택이나 명령어 해석기는 없습니다.

원본 데이터 형식의 부호와 래핑을 유지하기 위해 일부 상태는 `i16`입니다.
플레이 단계와 음악 종류는 enum, 입력은 bool 필드로 표현합니다. 충돌 격자는
40×24칸이며 한 칸은 8픽셀입니다. `0`은 빈칸, `1`은 벽/발판, `2`는 사다리,
`3`은 가시입니다. 아이템은 좌우 반쪽에 서로 다른 코드가 들어갑니다.

원본과의 비교를 통해 세부 동작을 유지했습니다. 예를 들어 걷는 방향을 바꾼
첫 틱에는 제자리에서 방향만 바뀌고, 적 생성 풀의 개수는 열 번째에서
0으로 돌아갑니다. 점프와 경계 판정의 순서도 원본을 따릅니다.

전체 실행 코드는 safe Rust이며, 외부 Cargo 의존성은 없습니다. WASM 공개 함수는
단일 스레드의 `RefCell<Engine>`을 통해 접근합니다. 프레임버퍼의 할당 주소와
WASM 메모리 크기를 고정하여 JavaScript의 이미지 뷰가 도중에 무효화되지 않게 합니다.
렌더링은 게임 상태와 리플레이 해시에 영향을 주지 않습니다.

### 에셋

`assets/`에 있는 게임 데이터는 `src/assets.rs`에서 `include_bytes!`로 읽어
WASM에 포함합니다.

| 파일 | 내용 |
| --- | --- |
| `PPK.B16` | 캐릭터 스프라이트, 마스크, 지형과 배경 타일 |
| `STAGE.PPK` | 전체 50개 스테이지의 맵 데이터 |
| `STAGE.PP1` | 타이틀 화면 이미지 |
| `STAGE.PP2` | 이어하기 화면 이미지 |
| `STAGE.PP3` | 게임 화면의 16색 VGA 팔레트 |
| `STAGE.PP4` | 사망 효과음 데이터 |
| `initial-data.bin` | 엔진이 초기 상태를 구성할 때 사용하는 게임 데이터 |

## 빌드 방법

- Python 3. 웹 파일 복사와 서비스 워커 생성에 표준 라이브러리를 사용합니다.
- Rust 개발 환경. `rustup`으로 Rust와 Cargo를 설치하고, 아래 명령으로 `wasm32-unknown-unknown` 타깃을 추가합니다.

빌드에 별도로 설치할 Python 패키지나 Rust 크레이트는 없습니다.
필요한 게임 데이터는 저장소에 포함되어 있습니다.

### Windows

저장소 루트에서 PowerShell을 열고 실행합니다.

```powershell
rustup target add wasm32-unknown-unknown
.\build.ps1
```

### macOS 또는 Linux

저장소 루트에서 실행합니다.

```sh
rustup target add wasm32-unknown-unknown
./build.sh
```

## 실행 방법

빌드하면 `dist/index.html`, `dist/port/ppk.wasm`, 서비스 워커 등 실행 파일이
`dist/`에 생성됩니다. 저장소 루트에서 로컬 HTTP 서버를 실행합니다.

- Windows PowerShell은 `py -3 -m http.server 8081 --bind 127.0.0.1 --directory dist`를 실행합니다. `py`가 없다면 `python -m http.server 8081 --bind 127.0.0.1 --directory dist`를 사용합니다.
- macOS 또는 Linux는 `python3 -m http.server 8081 --bind 127.0.0.1 --directory dist`를 실행합니다.

> [!NOTE]
> 스크립트는 Python 런처 `py`가 있으면 `py -3`를 사용하고, 없으면 `python` 또는 `python3`을 사용합니다.

브라우저에서 [http://127.0.0.1:8081/](http://127.0.0.1:8081/)을 엽니다.\
방향키로 이동하거나 사다리를 타고, Space로 시작하거나 점프합니다. P는 일시정지입니다.\
터치와 게임패드도 지원합니다.
