# 필요성 시각 대안 v12 검수

2026-10-15, v11 33장 통합본의 필요성 페이지를 기준으로 한 별도 5장 비교본.

다섯 페이지 전체 렌더링과 원본 크기 시각 검토 완료. 그림은 다섯 개의 별도 생성 결과이며 제목과 본문은 native 텍스트. 표나 Architecture 상자 도식으로 그림을 대체하지 않음. 구체적인 업무 대화 예시나 실측 수치 추가 없음.

파일 무결성, 5장 수량, 16:9 크기, 그림 관계, 도형 경계 및 heading fit 검사 통과. Calibri, 정수 pt와 ko-KR 위반 0건. 기준 v11 파일 해시 유지 확인. 일러스트와 사용 목적 및 해석 한계는 발표자 노트에 수록.

[Windows Open XML SDK 검사](https://github.com/cbyche/via-sw-architecture/actions/runs/38605153162) 통과. Windows Server 2022, Microsoft Open XML SDK 3.3.0 및 Office2021 스키마 기준으로 5장 파일 열기 성공과 오류 0건 확인. Windows의 검사 해시와 최종 로컬 PPTX 해시 일치. Calibri 설치 확인.

제작 commit `c6d0bb9d4d57f8fef8520a4209eaa0506d1141d7`의 [Architecture CI](https://github.com/cbyche/via-sw-architecture/actions/runs/38605135173) 통과. 상세 환경과 해시는 manifest에 기록.

PowerPoint 프로그램은 Windows cloud에 설치되어 있지 않아 자체 실행과 native 글꼴 배치는 미검증. 로컬 렌더링과 SDK 검증 범위로 한정.
