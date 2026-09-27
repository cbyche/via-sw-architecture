# DP-02 invalid v1 campaign

VIA-DP-02 v1은 QA-05 시작점 뒤에 per-trial SQLite connection·schema 준비 비용을
포함했고, QA-31 fault가 실제 connection 재개를 수행하지 않았다. 따라서 공식 근거로
사용하지 않는다. 수정된 endpoint와 실제 reopen recovery는 v2 campaign에서 실행한다.
