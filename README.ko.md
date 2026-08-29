# bit-context

> **AI 하네스 스킬을 위한 비트 메모리 컨텍스트 저장소** — 장황한 LLM 추론을 결정적 비트 연산으로 대체합니다.

## 해결하려는 문제

AI 하네스는 종종 LLM에게 수십 개의 불리언 조건을 평가하게 합니다:
- "사용자 인증됐는지, 권한 있는지, 쿼터 괜찮은지, 레이트리밋 괜찮은지, 리소스 존재하는지..."
- LLM이 모든 조건을 읽고, 각각 툴 호출하고, AND/OR 논리 추론
- **느림(초 단위), 비쌈(토큰), 비결정적(환각 위험)**

## 해결책

**bitctx**가 조건 평가를 LLM 밖으로 꺼냅니다:

```
┌─────────────┐     결정적      ┌──────────────┐     최소      ┌─────┐
│  Harness    │ ──── boolean ───► │   bitctx     │ ─── pass/fail ─► │ LLM │
│  (Python)   │   체크 (코드)     │  (비트 연산) │   + 실패 비트   │     │
└─────────────┘                   └──────────────┘                 └─────┘
        │                                   │                            │
        │  check_auth()                     │  0b1011 & 0b1111           │  "승인"
        │  check_rbac()                     │  = 0b1011 (통과)           │
        │  check_quota()                    │                            │
        ▼                                   ▼                            ▼
   코드가 각 조건                      비트 AND 연산                  사람용
   개별 판정                           O(1) 평가                      텍스트 생성
```

**결과**: 80%+ 토큰 절감, 50배 지연시간 개선, 결정적 의사결정.

---

## 아키텍처

| 구성요소 | 역할 |
|----------|------|
| **bitctx CLI** | 러스트 바이너리: 비트 메모리 + 마스크 평가 + 자연어 디코딩 |
| **스키마** | JSON: 비트 인덱스 ↔ 이름/설명, 네임드 마스크 (AND 조합) |
| **저장소** | `~/.bitctx/<session>.json` (파일 기반, 원자적 쓰기, 파일 락) |
| **스킬 래퍼** | `skills/bit-context/bitctx_skill.sh` 하네스 통합용 |

---

## 빠른 시작

```bash
# 빌드
cd bitctx-cli && cargo build --release

# 스키마 정의 (schema.json)
{
  "version": 1,
  "bits": {
    "0": {"name": "user_authenticated", "desc": "사용자 인증 완료"},
    "1": {"name": "has_permission", "desc": "필요 권한 보유"},
    "2": {"name": "quota_ok", "desc": "쿼터 초과 안 함"}
  },
  "masks": {
    "required": {"bits": [0, 1, 2], "desc": "필수 조건 모두"}
  }
}

# 세션 초기화
bitctx init --session deploy-123 --schema schema.json

# 하네스가 조건 확인 후 비트 설정
bitctx set --session deploy-123 --bit user_authenticated,has_permission --value true,true

# 즉시 비트 연산 평가
bitctx eval --session deploy-123 --mask required --format json
# {"pass":false,"missing":[2],"missing_labels":["quota_ok"]}

# 자연어 설명 (실패 시에만)
bitctx explain --session deploy-123 --mask required --lang ko
# "다음 조건이 충족되지 않았습니다: quota_ok"
```

---

## 하네스 통합 (Python)

```python
import subprocess, json, os

os.environ["BITCTX_SESSION"] = "task-123"
BITCTX = "/usr/local/bin/bitctx"

def bitctx_init(schema_path):
    subprocess.run([BITCTX, "init", "--session", "task-123", "--schema", schema_path], check=True)

def bitctx_set(bits: dict):
    names = ",".join(bits.keys())
    vals = ",".join("true" if v else "false" for v in bits.values())
    subprocess.run([BITCTX, "set", "--session", "task-123", "--bit", names, "--value", vals], check=True)

def bitctx_eval(mask: str) -> dict:
    result = subprocess.run([BITCTX, "eval", "--session", "task-123", "--mask", mask, "--format", "json"],
                           capture_output=True, text=True, check=True)
    return json.loads(result.stdout)

def bitctx_explain(mask: str) -> str:
    result = subprocess.run([BITCTX, "explain", "--session", "task-123", "--mask", mask, "--lang", "ko"],
                           capture_output=True, text=True, check=True)
    return result.stdout.strip()

# 사용 예시
bitctx_init("schema.json")
bitctx_set({"user_authenticated": True, "has_permission": True, "quota_ok": False})

result = bitctx_eval("required")
if result["pass"]:
    prompt = "배포 승인됨."
else:
    prompt = f"배포 거부됨: {bitctx_explain('required')}"
```

---

## 성능 비교

| 지표 | bitctx 미사용 | bitctx 사용 |
|------|---------------|-------------|
| 프롬프트 토큰 | ~450 | ~40 |
| LLM 툴 호출 | 12+ 개 | **0개** |
| 지연시간 | 2-10초 | **0.2-0.5초** |
| 결정적 | ❌ | **✅** |

자세한 비교는 [bench_harness.sh](bench_harness.sh) 참고.

---

## 프로젝트 구조

```
bit-mania/
├── specs/
│   └── goal-bitctx-cli.md       # 목표 및 요구사항 스펙
├── bitctx-cli/                  # 러스트 CLI
│   ├── src/
│   │   ├── models/              # Schema, Session
│   │   ├── storage/             # JSON I/O, 파일 락
│   │   └── commands/            # init, set, eval, explain, dump, reset
│   └── Cargo.toml
├── skills/bit-context/          # 하네스 스킬 래퍼
│   ├── bitctx_skill.sh
│   ├── example_schema.json
│   └── README.md
├── bench_perf.sh                # 마이크로벤치마크
├── bench_harness.sh             # 실제 하네스 비교
└── README.md                    # English version
```

---

## 로드맵

- [ ] v2: 데몬 모드 (Unix socket) - 서브밀리초 평가
- [ ] v2: 라이브러리 임베딩 (Rust crate / Python 바인딩)
- [ ] v2: 임의 비트 폭 지원 (bitvec)
- [ ] 스키마 마이그레이션 도구
- [ ] 비트 TTL/자동 만료

---

## 라이선스

MIT