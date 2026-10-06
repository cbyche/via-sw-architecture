# 발표용 요구사항 그림 생성 프롬프트

생성 방식: 내장 image_gen 도구. 각 항목을 별도 호출로 생성한다. 원본 PNG의 투명도를 유지한다.

## 공통 스타일

```text
Use case: productivity-visual.
Asset type: one standalone raster illustration for a Korean software architecture presentation.
Create one polished, compact explanatory illustration rather than a whole slide, diagram board or icon sheet. Match the spirit of the user's reference slide: clear professional flat illustration with a few concrete pictorial symbols, crisp dark navy outlines, soft filled shapes, restrained teal and sky-blue accents, tiny amber accents, very light volume and no dramatic shadows. Consistent stroke weight, rounded corners, minimal detail, legible when reduced to 250 pixels wide. Landscape composition approximately 4:3 with balanced empty padding around the whole artwork.
Background: genuinely transparent alpha. No text, letters, numbers, logos, watermark, title, border or decorative background. Do not imply a selected VIA architecture, particular component implementation, cloud deployment or measured performance. Only the described user capability or constraint.
```

## FR1: 지속 대화 처리

파일: `fr1-continuous-dialogue.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: A laptop conversation interface with a microphone and small keyboard input symbols at the left, an incoming teal speech waveform and typing bubble flowing toward the conversation, while an outgoing navy speech bubble and gentle sound waves simultaneously flow to the right. Show a small new user bubble entering even as the output exists. Both directions are clear and separate, representing accepting new input during an ongoing response. Exactly one laptop, no robot team, no disconnected input, no app screenshots with tiny UI text.
```

## FR2: 맥락 기반 요청 해석

파일: `fr2-contextual-interpretation.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: A laptop displaying a clean document with a simple table and a paragraph. A cursor highlights one bounded table region. Above it is a user speech bubble with an abstract pointer gesture. Add a small prior-conversation bubble and document thumbnail as contextual evidence. Visually connect the speech bubble and highlighted region to ONE neatly resolved request card with a target mark and check, using subtle connector lines. The focus is grounding an ambiguous user reference in actual screen and conversation context, not a magic brain inventing new material. No separated semantic modules or pipeline boxes.
```

## FR3: 복합 요청 처리

파일: `fr3-compound-request.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: One user speech bubble contains three small task pictograms, expanded into three separate document/task cards. A simple clear connector shows one card's result feeding the next; a small diamond fork with a check branch represents an explicit condition; a third short independent branch remains separate. Use a maximum of four task cards and one decision diamond, with crisp arrows on a compact horizontal composition. This is preservation of user-specified request relationships, not autonomous business planning. Use no labels, numbers, code, gear orchestra or complex flowchart.
```

## FR4: 다중 업무 상호작용

파일: `fr4-multiple-task-interaction.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: One central laptop chat interface connects to three clearly distinct work cards arranged in a compact fan, with separate colored lanes. One work card has a question bubble, one a progress indicator, and one a completed document result. A small user follow-up reply travels along the lane to the matching question card only. Avoid crossing or merging lanes; make matching request and result visible by repeated subtle shapes/colors. This represents one interaction window connecting multiple ongoing jobs without confusion, not multiple cloned AI models or a new actor architecture.
```

## FR5: 업무 연속성 유지

파일: `fr5-task-continuity.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: The same uniquely identifiable teal report/task card remains on one continuous curved line through three small pictorial situations: microphone connection switching off, a chat-window switch, and a laptop application restart shown by a circular restart arrow. At the end is that SAME single task card continuing with a small progress mark. Emphasize persistence of one job across connection and app changes; no duplicated job, no duplicated completed outputs, no cloud server, no perpetual/infinity guarantee. The line remains clear and the transient channel icons are secondary.
```

## FR6: 과거 맥락 활용

파일: `fr6-past-context.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: A tidy small archive of three history cards: a prior conversation bubble with a correction pencil, a completed report document, and a permitted preference represented by a small user profile card with a check. A magnifying glass selects a relevant history fragment, which is carried by a short arrow into one current request bubble beside a laptop. Show selective retrieval and reuse rather than dumping all history. Do not show a specific vector database, knowledge graph, central authoritative memory service or automatic permanent recording of every utterance.
```

## C1: 온디바이스 Omni 공유

파일: `c1-shared-on-device-omni.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: A laptop with a clear cutaway area within its screen/body containing exactly ONE central AI-style microchip, with a modest brain/circuit symbol inside and no lettering. A microphone/speech-wave symbol and a request-meaning symbol (a document with target mark) connect to that same chip via two short distinct lines, all inside or immediately bordering the laptop. Visually explicit one shared on-device model used for two roles; exactly one chip, no copied weights, no cloud, no second AI brain, no assertion that every possible helper is prohibited.
```

## C2: 업무 실행 위임

파일: `c2-downstream-work-delegation.png`

공통 스타일에 다음 프롬프트를 이어서 사용한다.

```text
Subject: A clear left-to-right delegation scene. On the left a laptop conversation interface connects a user's speech bubble to one prepared request envelope. The request envelope crosses a visible subtle dotted responsibility boundary via one arrow to a single worker/agent symbol on the right, where document creation, a tool/gear and an application action are grouped around that worker. A separate small completed-result arrow returns to the laptop. Keep the laptop interaction area free of tool/gear execution symbols; real work belongs on the worker side. The boundary is responsibility, not a cloud/network or necessarily remote deployment. No text, no multiple agents, no additional AI chip.
```
