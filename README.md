# VIA Software Architecture

Samsung PC용 Voice Interaction Agent(VIA)의 소프트웨어 아키텍처를 정의하고, Decision Point별 대안을 검증하기 위한 저장소다.

## Current source of truth

현재 기준선은 [`docs/architecture/`](docs/architecture/README.md)다. 시스템 범위, 대표 Use Case, 품질 속성, 측정 계약, Decision Point를 이 순서로 관리한다.

```text
System definition and use cases
    -> quality attributes and measurement contracts
    -> Decision Point A/B alternatives
    -> controlled prototype and benchmark
    -> evidence
    -> ADR
```

현재 Voice responsiveness 정의는 다음 세 지표로 분리한다.

- W-01: Agent delegation 결과를 사용자에게 Voice로 전달하기까지의 VIA 책임시간
- W-02: VIA direct Voice response의 전체 반응시간
- W-03: Agent progress/status가 제공 가능해진 뒤 Voice feedback이 들리기까지의 반응시간

정확한 endpoint와 포함·제외 구간은 [`voice-responsiveness.md`](docs/architecture/08-quality-attributes/voice-responsiveness.md)가 유일한 active source of truth다. 새 정의에 맞는 측정 코드는 아직 구현하지 않았으며, 기존 W12-G1 수치와 harness는 historical evidence다.

## Repository map

| 경로 | 역할 |
| --- | --- |
| [`docs/architecture/`](docs/architecture/README.md) | 현재 Architecture 기준선 |
| [`docs/adr/`](docs/adr/) | 채택된 Architecture Decision Record |
| [`docs/references/`](docs/references/) | 외부·내부 참조자료 |
| [`docs/archive/`](docs/archive/README.md) | v1.1, vNext/DP-00, W12-G1 과거 세대 |
| [`benchmark/architecture/`](benchmark/architecture/README.md) | 다음 측정 구현의 active 위치 |
| [`benchmark/archive/`](benchmark/archive/README.md) | 과거 benchmark 구현 |
| [`prototypes/gate2/`](prototypes/gate2/README.md) | 현재 Gate 2 후보 구현 |
| [`results/gate2/`](results/gate2/README.md) | 현재 및 과거 Gate 2 evidence |
| [`scripts/gate2/`](scripts/gate2/README.md) | 현재 검증 도구 |

## Historical generations

`requirements-v1.1`, `requirements-vNext`와 기존 DP-00/QA 실험은 삭제하지 않았다. active namespace에서 분리해 [`docs/archive/`](docs/archive/README.md)와 Git `archive/*` 태그로 보존한다. 과거 문서는 provenance와 참고용이며 현재 요구사항·지표·결과로 인용하지 않는다.

## Development

저장소 작업 규칙은 [`AGENTS.md`](AGENTS.md)와 [`CONTRIBUTING.md`](CONTRIBUTING.md)를 따른다. Python 검증은 `.venv`를 사용하고, Rust 후보 구현은 `prototypes/gate2`에서 검증한다.
