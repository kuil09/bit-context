# Bit Context Codex 스킬

이 디렉터리는 `bitctx` 외부에서 이미 검증된 불리언 조건을 결정적으로 평가하는 설치형 Codex 스킬입니다.

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
```

실제 근거가 있는 값만 설정하십시오. 통과 결과는 저장된 값이 선택한 마스크를 충족한다는 뜻뿐이며 외부 권한 승인이 아닙니다.

## 호환 래퍼

래퍼는 help를 제외한 모든 명령에 명시적 세션을 요구합니다.

```bash
export BITCTX_SESSION=task-123
./bitctx_skill.sh init example_schema.json
./bitctx_skill.sh eval required json
./bitctx_skill.sh init example_schema.json --force
./bitctx_skill.sh reset --force
```

선택 환경변수:

- `BITCTX_BIN`: 실행 파일 경로, 기본값 `bitctx`
- `BITCTX_DATA_DIR`: 격리 데이터 디렉터리, 기본값 `~/.bitctx`

세션 상태는 평문 JSON입니다. 비밀을 저장하지 마십시오.
