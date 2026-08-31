# bit-context

`bitctx`는 검증된 불리언 조건을 비트로 저장하고 이름 있는 AND 마스크를 결정적으로 평가하는 작은 Rust CLI입니다. 하네스가 각 조건을 확인하는 방법을 이미 알고 있고, 작고 지속적인 게이트 상태가 필요할 때 사용합니다.

[English documentation](README.md)

## 경계

`bitctx`는 사용자를 인증하거나, 사실을 발견하거나, 정책을 판단하거나, 권한을 부여하지 않습니다. 마스크 통과는 해당 세션에 저장된 값이 선택한 스키마 마스크를 충족한다는 뜻뿐입니다. 신뢰할 수 있는 조건값을 확보하고 외부 권한·정책 조건을 집행하는 책임은 호출자에게 있습니다.

상태는 평문 JSON으로 저장됩니다. 세션 ID, 비트 이름, 설명, 상태에 비밀을 넣지 마십시오.

## 설치

릴리스 지원 플랫폼은 x86-64와 ARM64 기반 Linux 및 macOS입니다. Windows는 현재 지원하지 않습니다.

최신 릴리스 자산을 필수 체크섬 검증과 함께 설치합니다.

```bash
curl -fsSL https://raw.githubusercontent.com/kuil09/bit-context/main/install.sh | bash
```

`/usr/local/bin` 이외의 위치에는 `INSTALL_DIR`을 지정하십시오. 체크섬 파일, 체크섬 도구, 자산 또는 다운로드를 사용할 수 없으면 설치기는 중단됩니다.

Rust 1.85 이상에서 소스로 빌드할 수도 있습니다.

```bash
cd bitctx-cli
cargo build --release --locked
install target/release/bitctx /usr/local/bin/bitctx
```

## 빠른 시작

스키마를 만듭니다.

```json
{
  "version": 1,
  "default_mask": "required",
  "bits": {
    "0": {"name": "user_authenticated", "desc": "인증 확인 완료"},
    "1": {"name": "has_permission", "desc": "필수 권한 확인 완료"},
    "2": {"name": "quota_ok", "desc": "쿼터 확인 통과"}
  },
  "masks": {
    "required": {"bits": [0, 1, 2], "desc": "모든 필수 조건"}
  }
}
```

명시적 세션을 초기화하고 평가합니다.

```bash
bitctx init --session deploy-123 --schema schema.json

# 새 세션은 모든 비트가 0이며 즉시 평가할 수 있습니다.
bitctx eval --session deploy-123 --mask required --format json

# 실제 검사에서 얻은 값만 설정합니다.
bitctx set --session deploy-123 \
  --bit user_authenticated,has_permission,quota_ok \
  --value true,true,false

bitctx eval --session deploy-123 --mask required --format json
```

실패 출력은 마스크 정의 순서를 유지하며 호환 필드와 구조화된 상세 정보를 함께 제공합니다.

```json
{
  "pass": false,
  "missing": [2],
  "missing_labels": ["quota_ok"],
  "missing_conditions": [
    {"index": 2, "name": "quota_ok", "desc": "쿼터 확인 통과"}
  ]
}
```

다른 대화나 에이전트에서 완료 절차를 다시 재생하지 않고 저장된 판단 상태를 복원합니다.

```bash
bitctx resume --session deploy-123 --format json
```

`resume`은 스키마의 선택형 `default_mask`를 사용하고, 없으면 유일한 마스크를 자동 선택합니다. 기본값 없이 마스크가 여러 개면 `--mask`를 명시해야 합니다. 저장된 체크포인트는 외부 근거가 여전히 최신임을 증명하지 않으므로 출력에는 `freshness: "unverified"`가 포함됩니다.
명령 성공은 상태를 읽고 평가했다는 뜻일 뿐입니다. 마스크 통과 여부는 `pass`를 파싱해 판단해야 합니다.

```json
{
  "session_id": "deploy-123",
  "schema_hash": "...",
  "mask": "required",
  "pass": false,
  "missing": [2],
  "missing_labels": ["quota_ok"],
  "missing_conditions": [
    {"index": 2, "name": "quota_ok", "desc": "쿼터 확인 통과"}
  ],
  "updated_at": "...",
  "freshness": "unverified"
}
```

빠르게 상태를 확인할 때는 텍스트 출력으로 모든 비트 위치를 고정 8×8 행렬에 표시할 수 있습니다.

```bash
bitctx eval --session deploy-123 --mask required --format text
```

```text
     0   1   2   3   4   5   6   7
00 ┌───┬───┬───┬───┬───┬───┬───┬───┐
   │ O │ O │ X │ · │ · │ · │ · │ · │
08 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
16 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
24 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
32 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
40 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
48 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
56 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
   └───┴───┴───┴───┴───┴───┴───┴───┘

RESULT: X
```

왼쪽 위가 bit 0이고 오른쪽 아래가 bit 63입니다. `O`는 선택된 조건이 충족됨, `X`는 현재 미충족, `·`는 선택한 마스크 밖의 위치를 뜻합니다. `X`는 검증된 거짓이라는 뜻이 아니며, 아직 근거가 없어 설정되지 않은 조건일 수도 있습니다.

텍스트 출력에 `--show all`, `--show satisfied`, `--show missing`을 추가하면 각각 전체, 충족, 미충족 조건의 이름과 설명을 마스크 정의 순서로 볼 수 있습니다. JSON 출력에서는 `--show`를 거부하며, JSON 기본값과 구조는 그대로 유지됩니다.

## 명령

```text
bitctx [--data-dir PATH] init    --session ID --schema FILE [--force]
bitctx [--data-dir PATH] set     --session ID --bit NAMES --value VALUES
bitctx [--data-dir PATH] eval    --session ID --mask NAME [--format json|text] [--show all|satisfied|missing]
bitctx [--data-dir PATH] resume  --session ID [--mask NAME] [--format json|text]
bitctx [--data-dir PATH] explain --session ID --mask NAME [--lang ko|en]
bitctx [--data-dir PATH] dump    --session ID [--format json|text]
bitctx [--data-dir PATH] reset   --session ID [--force]
```

데이터 디렉터리는 다음 우선순위로 선택됩니다.

1. `--data-dir <PATH>`
2. `BITCTX_DATA_DIR`
3. `~/.bitctx`

세션 ID는 `[A-Za-z0-9][A-Za-z0-9._-]{0,127}`과 일치하는 단일 정상 경로 컴포넌트여야 합니다. 경로 구분자, 절대경로, `.`, `..`, 제어문자, 더 긴 ID는 거부됩니다.

## 저장소와 동시성

기본 저장 구조는 다음과 같습니다.

```text
~/.bitctx/
├── .locks/
│   └── deploy-123.lock
└── deploy-123/
    ├── schema.json
    └── session.json
```

- `init`, `set`, `reset`은 세션별 배타 잠금을 사용합니다.
- `eval`, `resume`, `explain`, `dump`는 세션별 공유 잠금을 사용합니다.
- 잠금 파일은 삭제 가능한 세션 디렉터리 밖에 있습니다.
- 다른 프로세스가 같은 세션 ID를 참조할 때 잠금 inode가 바뀌는 경쟁을 막기 위해 `reset` 뒤에도 작은 잠금 파일이 남을 수 있습니다.
- 상태 쓰기는 같은 디렉터리의 임시파일을 flush·sync한 뒤 원자적으로 rename합니다.
- Unix에서 데이터 디렉터리는 `0700`, 상태와 잠금 파일은 `0600`을 사용합니다.
- 초기화되지 않은 세션에서 `set`은 실패합니다.
- `init --force`는 잠금을 유지한 채 스키마를 다시 저장하고 모든 비트를 0으로 초기화합니다.

스키마 검증은 중복 JSON 인덱스, 중복 비트 이름, 잘못된 이름, 존재하지 않는 마스크 참조, 존재하지 않는 `default_mask`, 빈 마스크, 마스크 내부의 중복 비트를 거부합니다. 설명에는 Unicode를 사용할 수 있습니다.

## v0.2 마이그레이션

유효한 v0.1 `schema.json`과 `session.json`은 그대로 읽을 수 있으며 스키마 해시 알고리즘도 유지됩니다. 주요 동작 변경은 다음과 같습니다.

- `init`이 두 파일과 0으로 초기화된 세션을 함께 만들어 직후 `eval`이 동작합니다.
- 호환 래퍼는 더 이상 `BITCTX_SESSION=default`를 사용하지 않습니다. 세션을 명시해야 합니다.
- `set`은 없는 세션을 만들지 않습니다.
- 안전하지 않은 세션 ID는 거부됩니다.
- `eval`은 `missing`과 `missing_labels`를 유지하면서 `missing_conditions`를 추가합니다.

## Codex 스킬

릴리스의 `bit-context-skill.zip`에는 `skills/bit-context/SKILL.md`, `agents/openai.yaml`, 호환 래퍼, 예제 스키마가 들어 있습니다. `bit-context` 디렉터리를 Codex 스킬 디렉터리에 풀고 스킬 발견을 다시 실행하십시오.

스킬은 `bitctx` 설치 여부를 확인하지만 자동 설치하지 않습니다. 관찰된 근거가 있는 조건값만 설정하며, 마스크 통과를 외부 권한 승인으로 해석하지 않습니다. 기존 작업이 이어질 때는 먼저 같은 세션을 평가하고, 변경되지 않은 true 비트를 완료된 체크포인트로 취급하며, 새 작업·변경된 비트·남은 조건만 보고합니다. 새 컨텍스트에서 알려진 세션 ID를 받으면 과거 대화를 요청하거나 완료 작업을 재구성하기 전에 `resume`을 실행합니다.

래퍼는 선택 사항입니다.

```bash
export BITCTX_SESSION=deploy-123
skills/bit-context/bitctx_skill.sh eval required json
skills/bit-context/bitctx_skill.sh resume
skills/bit-context/bitctx_skill.sh eval required text missing
```

## 성능 근거

`bench_perf.sh`는 실행한 머신의 로컬 CLI 프로세스 및 평가 시간을 측정합니다. `bench_harness.sh`의 토큰·모델 지연·비용 수치는 하네스 비교를 설명하기 위한 예시이며 로컬 CLI 측정값이나 보장값이 아닙니다.

```bash
BITCTX=bitctx ITERATIONS=100 ./bench_perf.sh
```

## 개발 검증

```bash
cd bitctx-cli
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked

cd ..
bash tests/test_wrapper.sh
shellcheck install.sh bench_perf.sh bench_harness.sh \
  skills/bit-context/bitctx_skill.sh tests/test_installer.sh \
  tests/smoke_release.sh tests/test_wrapper.sh
```

## 라이선스

MIT
