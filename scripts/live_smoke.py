"""Opt-in real-provider smoke test. Uses only synthetic, temporary workspaces.

Build first, then run:
  python scripts/live_smoke.py --binary target/release/mote --output smoke-results
No third-party Python packages. Exit 0 requires independently valid artifacts
from EVERY attempt; failed attempts are retained rather than hidden by retries.
"""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
import time
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', required=True, help='New evidence directory; must not exist')
    parser.add_argument('--endpoint', default='http://127.0.0.1:47113/v1/chat/completions')
    parser.add_argument('--model', default='auto')
    parser.add_argument('--auth-env', help='Name of an existing API-key environment variable')
    parser.add_argument('--trials', type=int, default=3)
    args = parser.parse_args()
    binary = Path(args.binary).resolve(strict=True)
    if args.trials < 1:
        parser.error('--trials must be positive')
    evidence = Path(args.output).resolve()
    evidence.mkdir(parents=True, exist_ok=False)
    results = []
    for trial in range(1, args.trials + 1):
        nonce = uuid.uuid4().hex
        total = sum([17 + trial, 29 + trial, 8 + trial])
        cases = [
            ('csv', {'orders.csv': f'status,amount\napproved,{17+trial}\nrejected,1000\napproved,{29+trial}\napproved,{8+trial}\n', 'nonce.txt': nonce},
             'Read orders.csv and nonce.txt. Write result.json with approved_total equal to the sum of amounts of approved rows only, and nonce copied exactly from nonce.txt.',
             'result.json', {'approved_total': total, 'nonce': nonce}),
            ('copy', {'input.txt': f'    indented {nonce}\nsecond line: caf\u00e9\n\n'},
             'Copy input.txt to copy.txt exactly, preserving all spaces, Unicode characters and newlines.',
             'copy.txt', f'    indented {nonce}\nsecond line: caf\u00e9\n\n'),
            ('config', {'config.json': json.dumps({'name': nonce, 'enabled': False, 'port': 8080})},
             'Read config.json. Write updated.json with enabled set to true and port set to 9090. Preserve the name field exactly and do not add fields.',
             'updated.json', {'name': nonce, 'enabled': True, 'port': 9090}),
        ]
        for name, inputs, task, output_name, expected in cases:
            case_name = f'{name}-{trial}'
            with tempfile.TemporaryDirectory(prefix='mote-live-') as tmp:
                root = Path(tmp)
                workspace = root / 'workspace'
                workspace.mkdir()
                for filename, text in inputs.items():
                    (workspace / filename).write_bytes(text.encode('utf-8'))
                model = {'provider': 'openai-compatible', 'model': args.model,
                         'endpoint': args.endpoint, 'timeout_seconds': 60}
                if args.auth_env:
                    model['auth_env'] = args.auth_env
                manifest = {'name': case_name, 'workspace': str(workspace),
                            'capabilities': ['read_file', 'write_file'],
                            'max_iterations': 8, 'max_runtime_seconds': 180,
                            'output_file': output_name, 'models': [model]}
                path = root / 'manifest.yaml'
                path.write_text(json.dumps(manifest), encoding='utf-8')
                start = time.monotonic()
                record = {'case': case_name, 'task': task, 'inputs': inputs, 'expected': expected}
                try:
                    proc = subprocess.run([str(binary), '--jsonl', str(path), task],
                                          capture_output=True, text=True, encoding='utf-8', timeout=200)
                    record.update(exit_code=proc.returncode, stdout=proc.stdout, stderr=proc.stderr)
                except subprocess.TimeoutExpired:
                    record.update(exit_code=None, error='external 200-second deadline exceeded')
                target = workspace / output_name
                content = target.read_bytes().decode('utf-8') if target.is_file() else None
                record['content'] = content
                try:
                    actual = json.loads(content) if isinstance(expected, dict) and content is not None else content
                except ValueError:
                    actual = None
                record['passed'] = record['exit_code'] == 0 and actual == expected
                record['seconds'] = round(time.monotonic() - start, 3)
                (evidence / f'{case_name}.json').write_text(json.dumps(record, indent=2), encoding='utf-8')
                results.append(record)
                print(f'{case_name}: {"PASS" if record["passed"] else "FAIL"} ({record["seconds"]}s)', flush=True)
    summary = {'attempts': len(results), 'passed': sum(r['passed'] for r in results),
               'binary': str(binary), 'endpoint': args.endpoint, 'model': args.model,
               'cases': [{'case': r['case'], 'passed': r['passed']} for r in results]}
    (evidence / 'summary.json').write_text(json.dumps(summary, indent=2), encoding='utf-8')
    print(json.dumps(summary), flush=True)
    return 0 if all(r['passed'] for r in results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
