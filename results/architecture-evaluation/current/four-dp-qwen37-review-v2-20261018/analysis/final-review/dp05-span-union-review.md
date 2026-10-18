# DP05 union-span 독립 읽기 검토

핵심 수정은 aggregate source_spans의 실제 공급 범위 포함 검사에 한정된다. 정렬한 원문 범위가 요청 시작부터 끝까지 빈틈없이 덮을 때만 허용하며, 각 범위의 source·현재 revision·현재 권한·철회를 다시 검사한다. 개별 fact quote 검사는 변경하지 않는다.

unit 생략과 명시 nonstring/null이 같은 fallback으로 처리되는 타입 경계 1건을 owner에게 전달했다. acceptance 실행은 모델 없는 준비 창 이후 owner가 수행하며 이 문서는 static 검토만 기록한다.

세부 SHA 및 범위: 같은 이름 JSON.

최종 source SHA `1220ef9747bfb302885fcd2e075aca5d27d561128bb3272182e85da8ed7647ca`에서 unit 생략만 implicit으로 허용하고 명시 byte_offset/17/true/null을 결과·공급 범위 모두 거절하는 guard를 정적으로 재확인하여 타입 경계 지적을 닫았다. 실제 cargo/actualwire 수용은 owner 기록 및 별도 controls 수용을 참조한다.
