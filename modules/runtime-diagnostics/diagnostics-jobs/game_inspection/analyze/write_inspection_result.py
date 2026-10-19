"""Publish one inspection report as JSON or standard output."""
import json

def write_result(result, output):
    text = json.dumps(result, indent=2, ensure_ascii=False) + '\n'
    if output:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text, encoding='utf-8')
        print(output)
    else:
        print(text)
