---
id: goal-bitctx-cli
type: goal-and-requirements
status: draft
title: "bitctx CLI: Bit-Memory Context Store for AI Harness Skills"
tags: [cli, rust, bitwise, context, harness-skill]
---

## Goal

AI 하네스(오케스트레이터)가 복잡한 조건 판단을 **자연어 프롬프트 체인 대신 비트 연산**으로 수행할 수 있도록, **CLI 도구(bitctx)**와 **하네스 스킬 래퍼**를 제공한다.

- 하네스 코드(결정적 런타임)가 조건을 평가해 비트 설정
- 비트 마스크로 **즉시 가부 판단** (O(1) 비트 연산)
- 실패 시에만 **자연어 디코딩**으로 이유 설명
- LLM 프롬프트에는 판단 결과만 전달 → **토큰/지연시간 대폭 절감**

## Scope

### In Scope

| 영역 | 내용 |
|------|------|
| **CLI 핵심 명령** | `init`, `set`, `eval`, `explain`, `dump`, `reset` |
| **비트 메모리 모델** | 최대 64비트(u64), 스키마 기반 이름↔위치 매핑 |
| **마스크 정의** | 스키마 내 named masks (AND 조합), `eval` 시 마스크명 참조 |
| **영속성** | 세션별 JSON 파일 (`~/.bitctx/<session>.json`) |
| **스키마** | JSON 파일로 정의 (bits, masks, descriptions) |
| **출력 포맷** | JSON (머신 파싱), 인간可读 텍스트 (디버깅) |
| **하네스 스킬 래퍼** | 쉘 래퍼 스크립트/함수로 기존 스킬 시스템과 통합 |

### Out of Scope (v1)

- 64비트 초과 조건 (u128/BitVec 확장 — v2)
- 분산/멀티프로세스 동기화 (단일 CLI 프로세스 전제)
- 네트워크 데몬 모드 (Unix socket/HTTP — v2)
- 조건 퍼지/확률적 평가 (불리언만)
- 스키마 자동 생성/마이그레이션 (수동 버저닝)
- GUI/TUI 대시보드

## Requirements

### Functional Requirements

| ID | 요구사항 | 우선순위 |
|----|----------|----------|
| FR-01 | `bitctx init --session <id> --schema <path>`: 세션 파일 생성, 스키마 검증 후 복사 | P0 |
| FR-02 | `bitctx set --session <id> --bit <name|index> --value <bool>`: 단일 비트 설정 | P0 |
| FR-03 | `bitctx set --session <id> --bits <csv> --values <csv>`: 다중 비트 일괄 설정 | P0 |
| FR-04 | `bitctx eval --session <id> --mask <name>`: 마스크 평가 → `{pass, missing[], missing_labels[]}` JSON 출력 | P0 |
| FR-05 | `bitctx explain --session <id> --mask <name> [--lang <ko|en>]`: 실패 이유 자연어 출력 | P0 |
| FR-06 | `bitctx dump --session <id> [--format json|text]`: 전체 비트 상태 + 스키마 매핑 출력 | P0 |
| FR-07 | `bitctx reset --session <id>`: 세션 파일 삭제 | P0 |
| FR-08 | 스키마 파일: `bits` (index→{name,desc}), `masks` (name→{bits[], desc}) 정의 | P0 |
| FR-09 | 세션 파일 자동 생성/로드 (`~/.bitctx/<session>.json`), 없으면 에러 | P0 |
| FR-10 | 하네스 스킬 예제: `skills/bit-context/` 디렉토리에 래퍼 제공 | P1 |

### Non-Functional Requirements

| ID | 요구사항 | 메트릭 |
|----|----------|--------|
| NFR-01 | 단일 `eval` 명령 실행 시간 | < 5ms (콜드 스타트 포함 < 20ms) |
| NFR-02 | 바이너리 크기 | < 5MB (stripped) |
| NFR-03 | 의존성 최소화 | `serde`, `serde_json`, `clap`, `anyhow` 외 최소 |
| NFR-04 | 크로스 플랫폼 | Linux, macOS, Windows (x86_64, aarch64) |
| NFR-05 | 에러 메시지 | 사용자 행동 가능한 형태 (스키마 불일치 시 비트 이름 명시) |

## Constraints & Assumptions

| 구분 | 내용 |
|------|------|
| **언어** | Rust (2021 edition, MSRV 1.70+) |
| **영속성** | JSON 파일만 (SQLite/DB 미사용) |
| **세션 저장소** | `~/.bitctx/` 디렉토리, 파일명 `<session>.json` |
| **비트 폭** | u64 (최대 64개 조건) |
| **스키마 버전** | v1 고정, 필드 `version: 1` 필수 |
| **하네스 통합** | 쉘 래퍼 방식 (stdin/stdout JSON), 네이티브 플러그인 아님 |

## Success Criteria

1. **통합 테스트 통과**: 하네스 스킬에서 `bitctx` 호출 → 조건 설정 → 마스크 평가 → 결과 수신 전체 플로우 동작
2. **성능**: 1000회 `eval` 연속 실행 < 1초 (콜드 스타트 제외)
3. **토큰 절약 검증**: 동일 태스크에서 LLM 프롬프트 크기 70% 이상 감소 (수동 측정)
4. **스키마 진화**: 조건 10개 추가 시 스키마만 수정, CLI 코드 변경 불필요

## Risks & Mitigations

| 위험 | 완화 |
|------|------|
| JSON 파일 동시 쓰기 경쟁 | 파일 락(`fs2`/`fd-lock`) 또는 원자적 쓰기(임시파일+rename) |
| 스키마-세션 불일치 (버전 차이) | `init` 시 스키마 해시 저장, `eval` 시 검증 후 불일치면 에러 |
| 비트 인덱스 중복/충돌 | 스키마 로드 시 중복 인덱스 검증, 에러로 중단 |
| 하네스에서 세션 ID 관리 실수 | `--session` 필수 인자화, 환경변수 `BITCTX_SESSION` 폴백 지원 |

## Future Extensions (v2+)

- u128 / 임의 비트 폭 (`bitvec` 크레이트)
- 데몬 모드 (`bitctx serve --socket /tmp/bitctx.sock`)로 프로세스 스폰 오버헤드 제거
- 스키마 마이그레이션 도구 (`bitctx migrate`)
- 조건 TTL/자동 만료
- 원격 세션 동기화 (CRDT/이벤트 소싱)