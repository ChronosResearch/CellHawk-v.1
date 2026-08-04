import urllib.request
import json
import ssl
import sys

ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE

repo = 'sidthebuilder/CellHawk-v.1'
branch = 'dependabot/cargo/nalgebra-0.35.0'
url = f'https://api.github.com/repos/{repo}/commits/{branch}/check-runs'

req = urllib.request.Request(url)
req.add_header('Accept', 'application/vnd.github.v3+json')
try:
    with urllib.request.urlopen(req, context=ctx) as response:
        data = json.loads(response.read().decode())
        
    for run in data.get('check_runs', []):
        if run['conclusion'] == 'failure':
            print(f"FAILED CHECK RUN: {run['name']}")
            print(f"Output Title: {run['output'].get('title')}")
            print(f"Output Summary:\n{run['output'].get('summary')}")
            annotations_url = run['url'] + '/annotations'
            req_ann = urllib.request.Request(annotations_url)
            req_ann.add_header('Accept', 'application/vnd.github.v3+json')
            try:
                with urllib.request.urlopen(req_ann, context=ctx) as ann_resp:
                    anns = json.loads(ann_resp.read().decode())
                    for ann in anns:
                        print(f"Annotation: {ann['path']}:{ann['start_line']} - {ann['message']}")
            except Exception as e:
                print(f"Could not fetch annotations: {e}")
            print("-" * 40)
except Exception as e:
    print(f"Error: {e}")
