# Post-BASIC 실행 정본

실제 새 보완/진단 runner와 compact formatter의 단일 정본은 `benchmark/architecture/four_dp_poc_v2/post_basic/`의 4개 Python 파일입니다. 이 analysis 폴더의 같은 이름 준비본은 실행 전 provenance로만 보존합니다. 실제 모델 호출에 사용하지 않습니다. 각 새 phase는 benchmark 정본을 frozen/source로 복사하고 source SHA에 포함합니다. 줄 번호 진단 worker와 formatter는 같은 정본의 immutable phase snapshot에서 실행합니다.

- `tactics_control.py`: BASIC120 이후 A4/B4/A2 별도 phase 준비/실행.
- `line_number_diagnostic.py`: BASIC120+27 이후 ABC E4×3 입력표현 진단.
- `compact_source_lines.py`: 모델 전송 복사본만 번호화.
- `check_compact_source_lines.py`: 순수 변환/실제32request 읽기 검사.

BASIC freeze에 포함된 기존 파일은 변경하지 않았습니다.
