# Bit Context Codex 스킬

이 디렉터리는 `bitctx` 외부에서 이미 검증된 불리언 조건을 결정적으로 평가하는 설치형 Codex 스킬입니다. 완료된 절차를 다시 답습하지 않고 안정적인 체크포인트에서 장기 작업을 재개하는 데에도 사용할 수 있습니다.

[English guide](README.md)

## 구성

- `SKILL.md`: 에이전트 지침과 안전 경계
- `agents/openai.yaml`: Codex 발견 메타데이터
- `bitctx_skill.sh`: 선택형 호환 래퍼
- `example_schema.json`: 예제 스키마

## 사전 조건

`bitctx`를 별도로 설치하고 사용할 수 있는지 확인합니다.

```bash
command -v bitctx
bitctx --version
```

스킬과 래퍼는 바이너리를 자동 설치하지 않습니다.

## 스킬 설치

이 디렉터리를 Codex 스킬 디렉터리 아래의 `bit-context`로 복사하거나 압축 해제합니다. `SKILL.md`가 해당 디렉터리 루트에 있어야 합니다.

## 직접 CLI 워크플로

```bash
SESSION_ID=task-123
bitctx init --session "$SESSION_ID" --schema example_schema.json
bitctx set --session "$SESSION_ID" \
  --bit user_authenticated,has_permission \
  --value true,true
bitctx eval --session "$SESSION_ID" --mask required --format json
bitctx resume --session "$SESSION_ID" --format json
```

실제 근거가 있는 값만 설정하십시오. 통과 결과는 저장된 값이 선택한 마스크를 충족한다는 뜻뿐이며 외부 권한 승인이 아닙니다.

## 완료 절차를 반복하지 않고 재개하기

작업이 이어질 때는 같은 명시적 세션을 재사용합니다. 먼저 대상 마스크를 평가하고, 입력과 근거가 바뀌지 않은 true 비트는 완료된 체크포인트로 취급하며, 누락되거나 새로 무효화된 조건만 처리합니다.

```bash
bitctx eval --session "$SESSION_ID" --mask required --format json

# 새로운 관찰로 이전 체크포인트 하나가 무효화되었습니다.
bitctx set --session "$SESSION_ID" --bit quota_ok --value false
bitctx eval --session "$SESSION_ID" --mask required --format json
```

대화 전체를 비트로 인코딩하지 마십시오. 원문과 섬세한 추론은 `bitctx` 밖에 두고, 비트는 안정적이고 판단에 필요한 체크포인트에만 사용합니다. 재개할 때는 완료된 작업을 다시 요약하지 말고, 이번에 확인하거나 변경한 조건과 순서가 보존된 `missing_conditions`만 보고합니다.

새 대화·에이전트·fresh context에서는 알려진 세션 ID로 과거 대화를 재생하지 않고 저장된 판단 상태를 복원할 수 있습니다.

```bash
bitctx resume --session "$SESSION_ID" --format json
```

`resume`은 `default_mask`를 사용하고, 없으면 스키마의 유일한 마스크를 선택합니다. 기본값 없이 마스크가 여러 개면 `--mask`를 명시해야 합니다. `freshness` 필드는 항상 `unverified`입니다. 저장된 체크포인트를 복원할 뿐 외부 근거가 여전히 최신임을 증명하지는 않습니다.

항상 JSON의 `pass` 필드를 확인하십시오. `eval` 또는 `resume` 프로세스의 성공 종료는 평가가 실행됐다는 뜻이지 선택한 마스크가 통과했다는 뜻이 아닙니다.

사람이 빠르게 상태를 볼 때는 `--format text`를 사용합니다. bit 0부터 63까지 항상 8×8 행렬로 표시하며 `O`는 충족, `X`는 미충족, `·`는 선택한 마스크 밖을 뜻합니다. `X`가 검증된 거짓을 의미하지는 않습니다. `--show all`, `--show satisfied`, `--show missing`으로 상세 목록을 순서대로 볼 수 있습니다.

## 호환 래퍼

래퍼는 help를 제외한 모든 명령에 명시적 세션을 요구합니다.

```bash
export BITCTX_SESSION=task-123
./bitctx_skill.sh init example_schema.json
./bitctx_skill.sh eval required json
./bitctx_skill.sh resume
./bitctx_skill.sh eval required text missing
./bitctx_skill.sh init example_schema.json --force
./bitctx_skill.sh reset --force
```

선택 환경변수:

- `BITCTX_BIN`: 실행 파일 경로, 기본값 `bitctx`
- `BITCTX_DATA_DIR`: 격리 데이터 디렉터리, 기본값 `~/.bitctx`

세션 상태는 평문 JSON입니다. 비밀을 저장하지 마십시오.
