#!/usr/bin/env python3
"""Send a fixed PCM WAV to Alibaba Qwen3-Omni Realtime without exposing secrets."""

from __future__ import annotations

import argparse
import base64
import getpass
import json
from pathlib import Path
import subprocess
import time
import wave

import websocket


API_KEY_SERVICE = "via-dashscope-api-key"
WORKSPACE_ID_SERVICE = "via-dashscope-workspace-id"
DEFAULT_MODEL = "qwen3-omni-flash-realtime-2025-12-01"


def read_keychain(service: str) -> str:
    result = subprocess.run(
        [
            "security",
            "find-generic-password",
            "-a",
            getpass.getuser(),
            "-s",
            service,
            "-w",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    value = result.stdout.strip()
    if not value:
        raise RuntimeError(f"Keychain item is empty: {service}")
    return value


def read_pcm_wav(path: Path) -> bytes:
    with wave.open(str(path), "rb") as source:
        actual = (
            source.getnchannels(),
            source.getframerate(),
            source.getsampwidth(),
            source.getcomptype(),
        )
        expected = (1, 16_000, 2, "NONE")
        if actual != expected:
            raise ValueError(f"Expected mono 16 kHz PCM16 WAV; got {actual}")
        audio = source.readframes(source.getnframes())
    if not audio:
        raise ValueError("Input WAV contains no audio frames")
    return audio


def write_reply_wav(path: Path, audio: bytes) -> None:
    with wave.open(str(path), "wb") as target:
        target.setnchannels(1)
        target.setsampwidth(2)
        target.setframerate(24_000)
        target.writeframes(audio)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wav", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--model", default=DEFAULT_MODEL)
    args = parser.parse_args()

    api_key = read_keychain(API_KEY_SERVICE)
    workspace_id = read_keychain(WORKSPACE_ID_SERVICE)
    audio = read_pcm_wav(args.wav)
    args.out_dir.mkdir(parents=True, exist_ok=True)

    url = (
        f"wss://{workspace_id}.ap-southeast-1.maas.aliyuncs.com"
        f"/api-ws/v1/realtime?model={args.model}"
    )
    ws = websocket.create_connection(
        url,
        header=["Authorization: Bearer " + api_key],
        timeout=30,
    )

    started_ns = time.monotonic_ns()
    input_end_ns: int | None = None
    first_audio_ns: int | None = None
    first_text_ns: int | None = None
    transcript_parts: list[str] = []
    input_transcript = ""
    output_audio = bytearray()
    event_log: list[dict[str, object]] = []
    response_status = "unknown"

    def receive() -> dict[str, object]:
        event = json.loads(ws.recv())
        now_ns = time.monotonic_ns()
        event_log.append(
            {
                "type": event.get("type", "unknown"),
                "since_connect_ms": round((now_ns - started_ns) / 1_000_000, 3),
            }
        )
        if event.get("type") == "error":
            raise RuntimeError(f"Alibaba Realtime error: {event.get('error')}")
        return event

    try:
        while receive().get("type") != "session.created":
            pass
        ws.send(
            json.dumps(
                {
                    "type": "session.update",
                    "session": {
                        "modalities": ["text", "audio"],
                        "voice": "Cherry",
                        "input_audio_format": "pcm",
                        "output_audio_format": "pcm",
                        "instructions": (
                            "한국어로 한 문장만 간결하게 답하세요. "
                            "외부 도구나 검색을 사용하지 마세요."
                        ),
                        "turn_detection": None,
                        "input_audio_transcription": {
                            "model": "qwen3-asr-flash-realtime"
                        },
                    },
                },
                ensure_ascii=False,
            )
        )
        while receive().get("type") != "session.updated":
            pass

        for offset in range(0, len(audio), 3_200):
            chunk = audio[offset : offset + 3_200]
            ws.send(
                json.dumps(
                    {
                        "type": "input_audio_buffer.append",
                        "audio": base64.b64encode(chunk).decode("ascii"),
                    }
                )
            )
            time.sleep(len(chunk) / (16_000 * 2))

        input_end_ns = time.monotonic_ns()
        ws.send(json.dumps({"type": "input_audio_buffer.commit"}))
        while receive().get("type") != "input_audio_buffer.committed":
            pass
        ws.send(json.dumps({"type": "response.create"}))

        while True:
            event = receive()
            event_type = event.get("type")
            now_ns = time.monotonic_ns()
            if event_type == "response.audio.delta":
                if first_audio_ns is None:
                    first_audio_ns = now_ns
                output_audio.extend(base64.b64decode(str(event["delta"])))
            elif event_type in ("response.audio_transcript.delta", "response.text.delta"):
                if first_text_ns is None:
                    first_text_ns = now_ns
                transcript_parts.append(str(event.get("delta", "")))
            elif event_type == "response.audio_transcript.done":
                transcript = str(event.get("transcript", ""))
                if transcript:
                    transcript_parts = [transcript]
            elif event_type == "conversation.item.input_audio_transcription.completed":
                input_transcript = str(event.get("transcript", "")).strip()
            elif event_type == "response.done":
                response = event.get("response", {})
                if isinstance(response, dict):
                    response_status = str(response.get("status", "unknown"))
                break
    finally:
        ws.close()

    if not output_audio:
        raise RuntimeError("No output audio received")
    assert input_end_ns is not None
    reply_path = args.out_dir / "reply.wav"
    trace_path = args.out_dir / "trace.json"
    write_reply_wav(reply_path, bytes(output_audio))
    trace = {
        "evidence": "MEASURED_MODEL",
        "model": args.model,
        "input_wav": str(args.wav),
        "input_audio_bytes": len(audio),
        "output_audio_bytes": len(output_audio),
        "response_status": response_status,
        "first_text_after_input_end_ms": (
            None
            if first_text_ns is None
            else round((first_text_ns - input_end_ns) / 1_000_000, 3)
        ),
        "first_audio_packet_after_input_end_ms": (
            None
            if first_audio_ns is None
            else round((first_audio_ns - input_end_ns) / 1_000_000, 3)
        ),
        "transcript": "".join(transcript_parts).strip(),
        "input_transcript": input_transcript,
        "events": event_log,
    }
    trace_path.write_text(
        json.dumps(trace, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(json.dumps({key: value for key, value in trace.items() if key != "events"}, ensure_ascii=False, indent=2))
    print(f"reply_wav={reply_path}")
    print(f"trace_json={trace_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
