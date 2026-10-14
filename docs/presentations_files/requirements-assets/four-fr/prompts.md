# 네 가지 기능 요구사항 아이콘 생성 기록

2026-10-14 내장 ImageGen 도구로 생성했다. 네 파일은 발표자료에 삽입할 수 있는 투명 배경 PNG이며, 문구는 이미지 밖에서 편집할 수 있도록 그림 안에 넣지 않았다. 이전 6개 FR의 그림과 파일을 보존하고 이 디렉터리를 별도로 사용한다.

## 공통 프롬프트

```text
Use case: productivity-visual.
Asset type: one standalone icon-like conceptual illustration for a Korean software architecture PowerPoint slide, readable at 240 pixels wide.
Style: polished minimal flat editorial pictogram, friendly rounded geometric silhouettes, dark charcoal thick uniform outlines, clean white object fills, restrained muted sage green and desaturated blue highlights, tiny warm amber emphasis only if useful. Spacious balanced square composition, one clear visual idea, substantial objects rather than tiny detail. Approximately 12% empty padding. True transparent background, no enclosing tile, no drop shadow, no gradient, no 3D, no texture, no watermark, no logo. No written words, no letters, no numbers. Question marks, check marks, waveform marks and simple UI lines are allowed as pictorial symbols. The interior of white objects must remain opaque. All four illustrations belong to one consistent icon family. Create only the requested single illustration, not a contact sheet.
```

## FR1. 대화 지속성 제공

파일: [fr1-dialogue-continuity.png](./fr1-dialogue-continuity.png)

```text
Primary request: show that a user can keep speaking and correct themselves even when an agent question or result arrives. A simple human bust on the left speaks into a large sage speech bubble containing a waveform and a small pencil correction symbol. A small friendly robot bust on the right has a blue speech bubble carrying a question mark with a small result card carrying a checkmark beside it. The two input bubbles partly overlap in the center but remain clearly distinct; a bold flowing curved arrow underneath carries the conversation forward to one small fresh speech bubble. Make the two simultaneous speakers and uninterrupted continuation immediately understandable. Limit small ornaments; do not use an infinity sign or a stop symbol.
```

## FR2. 음성-화면 지칭 연결

파일: [fr2-voice-screen-grounding.png](./fr2-voice-screen-grounding.png)

```text
Primary request: show spoken reference being connected to the exact pointed-at item on a computer screen. One large desktop monitor contains two simple visual objects, a small grey table and a blue bar chart. The bar chart alone is enclosed by a sage selection outline, with a large mouse pointer touching it. A floating speech bubble with a simple waveform sits near the monitor, and a short clear curved arrow from that bubble ends on the selected chart. Other screen content stays neutral. The intended meaning is 'this chart' matched to current screen pointing. No keyboard, no person, no decorative icons.
```

## FR3. 과거 이력 맥락 제공

파일: [fr3-history-context.png](./fr3-history-context.png)

```text
Primary request: show relevant past conversation and completed work being recalled to help understand a current request. On the left is a compact overlapping stack: one past speech-bubble card and one completed document with a checkmark; a small clock badge marks this pair as history. One short passage strip across a history card is highlighted sage. A single broad curved arrow carries a small sage excerpt card from this history stack toward a large fresh blue speech bubble on the right. The excerpt nests inside the new bubble like a useful contextual piece. Show selective reuse of past records, not a backup, not a calendar, not a brain. Simple large shapes, strong left-to-right reading.
```

## FR4. 대화-업무 관계 연결

파일: [fr4-dialogue-work-linkage.png](./fr4-dialogue-work-linkage.png)

```text
Primary request: show that one conversation remains linked to its request, clarification question, work task and external agent execution. Build a compact coherent chain/network of exactly five large pictograms: an overlapping pair of speech bubbles for the conversation, a small request document with two blank lines, a question-mark bubble, a work clipboard with two checklist ticks, and a friendly robot bust with a small play-triangle badge for agent execution. Connect them with thick rounded continuous lines, with the question bubble branching from the request between conversation and task; task is linked to robot. Use a broad shallow U or compact two-row arrangement to fit square space, maintain clear whitespace. Connections rather than arrowheads emphasize preserved relationships, not mandatory sequential processing. Use sage and blue node accents, no disconnected nodes, no database, no padlock, no gears, no abstract dots as substitutes for the five objects.
```
