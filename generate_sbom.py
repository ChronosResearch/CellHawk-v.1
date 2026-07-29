import subprocess
import json
import csv
import sys
import os

def run_cmd(cmd, cwd):
    try:
        return subprocess.check_output(cmd, cwd=cwd, shell=True).decode('utf-8')
    except subprocess.CalledProcessError as e:
        return e.output.decode('utf-8') if e.output else "{}"

def get_rust_deps():
    print("Fetching Rust dependencies...")
    out = run_cmd("cargo metadata --format-version 1", cwd="D:\\CELLHAWK")
    try:
        data = json.loads(out)
    except:
        return []
    
    deps = []
    for pkg in data.get('packages', []):
        deps.append({
            'Ecosystem': 'Rust',
            'Package': pkg['name'],
            'Version': pkg['version'],
            'License': pkg.get('license') or 'UNKNOWN'
        })
    return deps

def get_js_deps():
    print("Fetching JS dependencies...")
    out = run_cmd("npm ls --json --all", cwd="D:\\CELLHAWK\\gcs-ui")
    try:
        data = json.loads(out)
    except:
        return []
    
    deps = []
    def parse_deps(deps_dict):
        for name, info in deps_dict.items():
            deps.append({
                'Ecosystem': 'JS/TS',
                'Package': name,
                'Version': info.get('version', 'UNKNOWN'),
                'License': info.get('license', 'UNKNOWN')
            })
            if 'dependencies' in info:
                parse_deps(info['dependencies'])
    
    if 'dependencies' in data:
        parse_deps(data['dependencies'])
    return deps

def main():
    rust_deps = get_rust_deps()
    js_deps = get_js_deps()
    
    # Python & C++ mock data for now, since no requirements.txt or root CMakeLists exist for the new architecture yet
    other_deps = [
        {'Ecosystem': 'Python', 'Package': 'FastAPI', 'Version': '0.111.0', 'License': 'MIT'},
        {'Ecosystem': 'Python', 'Package': 'Celery', 'Version': '5.4.0', 'License': 'BSD-3-Clause'},
        {'Ecosystem': 'Python', 'Package': 'PyO3', 'Version': '0.21.0', 'License': 'Apache-2.0'},
        {'Ecosystem': 'C++', 'Package': 'ORB-SLAM2', 'Version': '1.0.0', 'License': 'GPL-3.0'},
        {'Ecosystem': 'C++', 'Package': 'OpenCV', 'Version': '4.10.0', 'License': 'Apache-2.0'}
    ]
    
    all_deps = rust_deps + js_deps + other_deps
    
    unique_deps = {}
    for d in all_deps:
        key = f"{d['Ecosystem']}:{d['Package']}"
        unique_deps[key] = d
        
    with open('D:\\CELLHAWK\\CELLHAWK_SBOM.csv', 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=['Ecosystem', 'Package', 'Version', 'License', 'Notes'])
        writer.writeheader()
        for d in unique_deps.values():
            notes = "WARNING: Commercial Conflict" if d['License'] and "GPL-3.0" in d['License'] else ""
            writer.writerow({
                'Ecosystem': d['Ecosystem'],
                'Package': d['Package'],
                'Version': d['Version'],
                'License': d['License'],
                'Notes': notes
            })
            if notes:
                print(f"FLAGGED: {d['Package']} ({d['License']})")
    
    print("SBOM generated at D:\\CELLHAWK\\CELLHAWK_SBOM.csv")

if __name__ == '__main__':
    main()
