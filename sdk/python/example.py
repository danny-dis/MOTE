import json
import os
from mote_sdk import Client, Tool

model = {
    "provider": "openai-compatible",
    "model": os.environ.get("MOTE_MODEL", "auto"),
    "endpoint": os.environ.get("MOTE_MODEL_ENDPOINT", "http://127.0.0.1:47113/v1/chat/completions"),
}
if os.environ.get("MODEL_API_KEY"):
    model["auth_env"] = "MODEL_API_KEY"
calls = []
result = Client(binary=os.environ.get("MOTE_BRIDGE", "mote-bridge")).run(
    {"name": "python-example", "capabilities": ["uppercase"], "models": [model]},
    'Call uppercase with {"text":"hello"}, then complete with its returned text.',
    {"uppercase": Tool("Input: text string. Returns uppercase text.", lambda data: calls.append(data) or data["text"].upper())},
)
observed = any(event.get("type") == "ObservationReceived" and event.get("data", {}).get("text") == "HELLO" for event in result.events)
if not result.success or not calls or not observed:
    raise RuntimeError("Example failed its independent callback/observation checks")
print(json.dumps({"state": result.state, "observation": "HELLO", "callback_used": bool(calls)}))
