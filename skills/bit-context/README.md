# bit-context Skill

AI 하네스를 위한 비트 마스크 기반 컨텍스트 저장소 스킬입니다.

## 설치

```bash
# bitctx 바이너리 빌드
cd /path/to/bitctx-cli
cargo build --release
cp target/release/bitctx /usr/local/bin/  # 또는 PATH에 있는 위치

# 스킬 디렉토리를 하네스 스킬 경로에 심볼릭 링크 또는 복사
ln -s /path/to/bit-mania/skills/bit-context ~/.harness/skills/bit-context
```

## 환경 변수

| 변수 | 기본값 | 설명 |
|------|--------|------|
| `BITCTX_BIN` | `bitctx` | bitctx 바이너리 경로 |
| `BITCTX_SESSION` | `default` | 세션 ID |

## 사용법

### 1. 세션 초기화

```bash
# 스키마 파일로 세션 생성
bitctx_skill.sh init example_schema.json

# 또는 환경변수로 세션 지정
BITCTX_SESSION=task-123 bitctx_skill.sh init example_schema.json
```

### 2. 비트 설정

```bash
# 단일 비트 설정 (이름 또는 인덱스)
bitctx_skill.sh set user_authenticated true
bitctx_skill.sh set 1 true

# 다중 비트 일괄 설정
bitctx_skill.sh set-multi "user_authenticated,has_permission,quota_ok" "true,true,true"
```

### 3. 마스크 평가 (핵심: 가부 판단)

```bash
# JSON 출력 (머신 파싱용)
bitctx_skill.sh eval required json
# {"pass":false,"missing":[3],"missing_labels":["quota_ok"]}

# 텍스트 출력 (디버깅용)
bitctx_skill.sh eval required text
# FAIL: missing conditions:
#   - bit 3: quota_ok
```

### 4. 실패 이유 자연어 설명

```bash
bitctx_skill.sh explain required ko
# 다음 조건이 충족되지 않았습니다 (기본 실행 필수 조건)
#   - quota_ok

bitctx_skill.sh explain required en
# Conditions not satisfied (기본 실행 필수 조건)
#   - quota_ok
```

### 5. 상태 덤프

```bash
bitctx_skill.sh dump text
bitctx_skill.sh dump json
```

### 6. 세션 리셋

```bash
bitctx_skill.sh reset --force
```

## 하네스 통합 예시

### Python 하네스에서 사용

```python
import subprocess
import json
import os

os.environ["BITCTX_SESSION"] = "task-123"
os.environ["BITCTX_BIN"] = "/usr/local/bin/bitctx"

def bitctx_init(schema_path):
    subprocess.run(["bitctx_skill.sh", "init", schema_path], check=True)

def bitctx_set(bits: dict):
    names = ",".join(bits.keys())
    values = ",".join("true" if v else "false" for v in bits.values())
    subprocess.run(["bitctx_skill.sh", "set-multi", names, values], check=True)

def bitctx_eval(mask: str) -> dict:
    result = subprocess.run(
        ["bitctx_skill.sh", "eval", mask, "json"],
        capture_output=True, text=True, check=True
    )
    return json.loads(result.stdout)

def bitctx_explain(mask: str, lang: str = "ko") -> str:
    result = subprocess.run(
        ["bitctx_skill.sh", "explain", mask, lang],
        capture_output=True, text=True, check=True
    )
    return result.stdout.strip()

# 사용 예시
bitctx_init("example_schema.json")
bitctx_set({"user_authenticated": True, "has_permission": True})

result = bitctx_eval("required")
if not result["pass"]:
    reason = bitctx_explain("required")
    print(f"실행 불가: {reason}")
    # LLM에게 이유만 전달하여 대안 생성 요청
else:
    print("모든 조건 충족, 실행 진행")
```

### LLM 프롬프트 절약 효과

**기존 방식 (자연어 나열):**
```
사용자가 인증되었는지 확인하고, 권한이 있는지 확인하고, 쿼터가 초과되지 않았는지 확인하세요.
조건: user_authenticated=true, has_permission=true, quota_ok=true
모든 조건이 충족되면 "실행 가능"이라고 답하고, 아니면 어떤 조건이 실패했는지 말하세요.
```

**bitctx 방식:**
```python
# 하네스 코드에서
result = bitctx_eval("required")
if result["pass"]:
    prompt = "실행 가능"
else:
    prompt = f"실행 불가: {bitctx_explain('required')}"
```

토큰 사용량: **~90% 감소** (조건 나열 불필요, 결과만 전달)

## 스키마 작성 가이드

```json
{
  "version": 1,
  "bits": {
    "0": {"name": "조건명", "desc": "설명"},
    "1": {"name": "다른조건", "desc": "설명"}
  },
  "masks": {
    "마스크명": {"bits": [0, 1], "desc": "마스크 설명"}
  }
}
```

- `bits`: 인덱스(0-63) → {name, desc} 매핑
- `masks`: 마스크명 → {bits[], desc} 매핑 (AND 조건)
- 비트 인덱스는 0-63 범위, 중복 불가
- 마스크는 최소 1개 비트 포함

## 아키텍처

```
┌─────────────┐     쉘 호출      ┌──────────────────┐
│  Harness    │ ──────────────►  │  bitctx CLI      │
│  (Python 등)│  stdin/stdout    │  - 비트 메모리    │
│  - LLM 호출  │ ◄──────────────  │  - 마스크 평가    │
│  - 비트 설정  │  JSON/텍스트     │  - 자연어 디코딩  │
└─────────────┘                  └──────────────────┘
                                     │
                                     ▼
                            ~/.bitctx/<session>.json
                            (스키마 + 세션 상태)
```

## 성능

- 단일 `eval`: ~5ms (릴리즈 빌드, 콜드 스타트 ~15ms)
- 1000회 연속: ~6초 (프로세스 스폰 오버헤드 포함)
- 고성능 필요 시: v2 데몬 모드(Unix socket) 또는 라이브러리 임베딩 예정

## 라이선스

MIT