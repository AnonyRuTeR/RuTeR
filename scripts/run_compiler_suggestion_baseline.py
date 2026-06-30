#!/usr/bin/env python3
"""
Compiler-suggestion-only baseline: apply rustc MachineApplicable + MaybeIncorrect
suggestions to all Phase 1A frozen attempts, measure repair rate.
"""
import json, os, sys, re, shutil, subprocess, tempfile, argparse
from pathlib import Path
from collections import defaultdict
from concurrent.futures import ProcessPoolExecutor, as_completed
from datetime import timezone

EVAL = Path(os.environ.get('EVAL_DIR', '.'))
CRATES = EVAL / 'crates'
RUG_RUNS = EVAL / 'rug_runs'
FROZEN = EVAL / 'phase_1A/frozen'

def load_attempt_manifest():
    with open(FROZEN / 'frozen_attempt_manifest.json') as f:
        return json.load(f)

def load_rug_code(run_id, node_key, attempt_index):
    """Extract the injected test code for a specific attempt from RUG detailed_log."""
    log_path = RUG_RUNS / run_id / 'detailed_log.json'
    if not log_path.exists():
        return None

    with open(log_path) as f:
        data = json.load(f)

    fn_data = data.get(node_key)
    if fn_data is None:
        # Try fuzzy match
        for k, v in data.items():
            if k.endswith(node_key) or node_key.endswith(k.rsplit('::', 1)[-1] if '::' in k else k):
                fn_data = v
                break

    if fn_data is None:
        return None

    tg = fn_data.get('test_generation', [])
    if not isinstance(tg, list):
        return None

    # Manifest 'attempt' is 1-indexed; convert to 0-indexed
    idx = max(0, int(attempt_index) - 1) if attempt_index else 0
    if idx >= len(tg):
        return None

    attempt = tg[idx]
    code = attempt.get('injected_code', '')
    if not code or not code.rstrip().endswith('}'):
        return None

    return code

def find_source_file(crate_path, fn_key):
    module = re.sub(r'<.*?>', '', fn_key.split('::')[0]).strip()
    if not module:
        for part in fn_key.split('::'):
            clean = re.sub(r'<.*?>', '', part).strip()
            if clean and not clean[0].isupper():
                module = clean
                break
    src = crate_path / 'src'
    for c in [src / f'{module}.rs', src / module / 'mod.rs', src / 'lib.rs']:
        if c.exists():
            return c
    return src / 'lib.rs'

def run_cargo_check(crate_dir):
    proc = subprocess.run(
        ['cargo', 'check', '--tests', '--message-format=json'],
        cwd=crate_dir, capture_output=True, text=True, timeout=180,
    )
    return proc.stdout, proc.stderr, proc.returncode

def parse_suggestions(stdout):
    """Extract MachineApplicable + MaybeIncorrect suggestions from cargo NDJSON output.

    cargo --message-format=json uses 'reason': 'compiler-message' with
    the diagnostic nested under 'message'. Also supports the older
    '$message_type': 'diagnostic' format.
    """
    suggestions = []
    errors = defaultdict(int)

    for line in stdout.split('\n'):
        if not line.strip():
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        reason = msg.get('reason', '')
        if reason != 'compiler-message':
            # Also try older format
            if msg.get('$message_type') != 'diagnostic':
                continue

        diag = msg.get('message', msg)
        if diag.get('level') != 'error':
            continue

        code = diag.get('code', {})
        if isinstance(code, dict):
            errors[code.get('code', 'NO_CODE')] += 1

        for child in diag.get('children', []):
            for span in child.get('spans', []):
                appl = span.get('suggestion_applicability', '')
                sr = span.get('suggested_replacement')
                if appl in ('MachineApplicable', 'MaybeIncorrect') and sr:
                    suggestions.append({
                        'file_name': span.get('file_name', ''),
                        'byte_start': span.get('byte_start', 0),
                        'byte_end': span.get('byte_end', 0),
                        'replacement': sr,
                        'applicability': appl,
                    })

    return dict(errors), suggestions

def apply_suggestions_to_workspace(workspace_dir, suggestions):
    """Apply suggestions to files in workspace. Right-to-left to preserve offsets."""
    by_file = defaultdict(list)
    for s in suggestions:
        fname = Path(s['file_name']).name
        for src_file in (workspace_dir / 'src').iterdir():
            if src_file.is_file() and src_file.name == fname:
                by_file[src_file].append(s)
                break
        # Also check subdirectories
        for subdir in (workspace_dir / 'src').iterdir():
            if subdir.is_dir():
                for src_file in subdir.iterdir():
                    if src_file.is_file() and src_file.name == fname:
                        by_file[src_file].append(s)

    total_applied = 0
    for src_file, sugs in by_file.items():
        with open(src_file) as f:
            source = f.read()

        sugs.sort(key=lambda x: -x['byte_start'])
        for s in sugs:
            bs, be, sr = s['byte_start'], s['byte_end'], s['replacement']
            if 0 <= bs <= be <= len(source):
                source = source[:bs] + sr + source[be:]
                total_applied += 1

        with open(src_file, 'w') as f:
            f.write(source)

    return total_applied

def process_one_attempt(attempt_entry):
    """Process a single frozen attempt with the compiler-suggestion-only protocol."""
    uid = attempt_entry['attempt_uid']
    run_id = attempt_entry.get('run_id', '')
    node_key = attempt_entry.get('node_id', attempt_entry.get('node_key', ''))
    attempt_index = str(attempt_entry.get('attempt', attempt_entry.get('attempt_index', '0')))
    crate_name = attempt_entry.get('crate', 'humantime')
    src_path = attempt_entry.get('src_path', 'src/lib.rs')

    result = {
        'attempt_uid': uid,
        'batch_id': attempt_entry.get('batch_id', uid),
        'status': 'error',
        'errors_before': {}, 'errors_after': {},
        'suggestions_found': 0, 'suggestions_applied': 0,
        'MachineApplicable': 0, 'MaybeIncorrect': 0,
        'fixed': False, 'improved': False,
        'error': '',
    }

    # Get test code
    code = load_rug_code(run_id, node_key, attempt_index)
    if code is None:
        result['error'] = 'failed to load test code'
        return result

    clean_crate = CRATES / crate_name
    if not clean_crate.exists():
        result['error'] = f'crate {crate_name} not found'
        return result

    # Create workspace
    tmp = Path(tempfile.mkdtemp(prefix=f'cs_{crate_name}_'))
    try:
        shutil.copytree(clean_crate, tmp, dirs_exist_ok=True)
    except Exception as e:
        result['error'] = f'copy failed: {e}'
        return result

    # Find target file and inject test code
    target_file = tmp / src_path
    if not target_file.exists():
        # Fallback: use fn_key to find file
        target_file = find_source_file(tmp, node_key)

    with open(target_file, 'a') as f:
        f.write('\n' + code + '\n')

    # Run cargo check
    stdout, stderr, rc = run_cargo_check(tmp)
    if rc == 0:
        # Test already compiles! (shouldn't happen for frozen failed attempts)
        result['status'] = 'already_compiles'
        result['errors_before'] = {}
        result['errors_after'] = {}
        shutil.rmtree(tmp, ignore_errors=True)
        return result

    # Parse suggestions
    errors_before, suggestions = parse_suggestions(stdout)
    result['errors_before'] = errors_before

    if not suggestions:
        result['status'] = 'no_suggestions'
        shutil.rmtree(tmp, ignore_errors=True)
        return result

    result['suggestions_found'] = len(suggestions)
    result['MachineApplicable'] = sum(1 for s in suggestions if s['applicability'] == 'MachineApplicable')
    result['MaybeIncorrect'] = sum(1 for s in suggestions if s['applicability'] == 'MaybeIncorrect')

    # Apply suggestions
    n_applied = apply_suggestions_to_workspace(tmp, suggestions)
    result['suggestions_applied'] = n_applied

    if n_applied == 0:
        result['status'] = 'no_applicable_suggestions'
        shutil.rmtree(tmp, ignore_errors=True)
        return result

    # Re-run cargo check
    stdout_after, stderr_after, rc_after = run_cargo_check(tmp)
    errors_after, _ = parse_suggestions(stdout_after)
    result['errors_after'] = errors_after

    before_total = sum(errors_before.values())
    after_total = sum(errors_after.values())

    if after_total == 0:
        result['status'] = 'fixed'
        result['fixed'] = True
        result['improved'] = True
    elif after_total < before_total:
        result['status'] = 'improved'
        result['improved'] = True
    else:
        result['status'] = 'unchanged'

    shutil.rmtree(tmp, ignore_errors=True)
    return result


def main():
    parser = argparse.ArgumentParser(description='Compiler-suggestion-only baseline')
    parser.add_argument('--max', type=int, default=0, help='Max attempts to process (0=all)')
    parser.add_argument('--workers', type=int, default=4, help='Parallel workers')
    parser.add_argument('--out', type=str, default='', help='Output directory')
    args = parser.parse_args()

    fam = load_attempt_manifest()
    attempts = fam.get('attempts', [])

    if args.max > 0:
        attempts = attempts[:args.max]

    print(f'Processing {len(attempts)} attempts with {args.workers} workers...')

    out_dir = Path(args.out) if args.out else EVAL / 'phase_1A' / 'artifacts-cs'
    os.makedirs(out_dir, exist_ok=True)

    results = []
    stats = {
        'total': 0, 'fixed': 0, 'improved': 0, 'unchanged': 0,
        'no_suggestions': 0, 'already_compiles': 0, 'error': 0,
        'errors_before_total': 0, 'errors_after_total': 0,
        'suggestions_total': 0, 'suggestions_applied_total': 0,
        'machine_total': 0, 'maybe_total': 0,
    }

    # Process sequentially with progress
    for i, entry in enumerate(attempts):
        r = process_one_attempt(entry)
        results.append(r)

        stats['total'] += 1
        if r['fixed']: stats['fixed'] += 1
        if r['improved']: stats['improved'] += 1
        if r['status'] == 'unchanged': stats['unchanged'] += 1
        if r['status'] == 'no_suggestions': stats['no_suggestions'] += 1
        if r['status'] == 'already_compiles': stats['already_compiles'] += 1
        if r['status'] == 'error': stats['error'] += 1
        stats['errors_before_total'] += sum(r['errors_before'].values())
        stats['errors_after_total'] += sum(r['errors_after'].values())
        stats['suggestions_total'] += r['suggestions_found']
        stats['suggestions_applied_total'] += r['suggestions_applied']
        stats['machine_total'] += r['MachineApplicable']
        stats['maybe_total'] += r['MaybeIncorrect']

        if (i + 1) % 50 == 0:
            pct = (i + 1) / len(attempts) * 100
            print(f'  [{i+1}/{len(attempts)} {pct:.0f}%] fixed={stats["fixed"]} improved={stats["improved"]} no_sug={stats["no_suggestions"]} unchanged={stats["unchanged"]} err={stats["error"]}')

    # Save results
    strict_rate = stats['fixed'] / stats['total'] * 100 if stats['total'] else 0
    improved_rate = stats['improved'] / stats['total'] * 100 if stats['total'] else 0

    summary = {
        'baseline': 'compiler_suggestion_only',
        'total': stats['total'],
        'fixed': stats['fixed'],
        'fixed_rate': round(stats['fixed'] / stats['total'], 4) if stats['total'] else 0,
        'improved': stats['improved'],
        'improved_rate': round(stats['improved'] / stats['total'], 4) if stats['total'] else 0,
        'no_suggestions': stats['no_suggestions'],
        'already_compiles': stats['already_compiles'],
        'errors': stats['error'],
        'suggestions_total': stats['suggestions_total'],
        'suggestions_applied': stats['suggestions_applied_total'],
        'machine_applicable': stats['machine_total'],
        'maybe_incorrect': stats['maybe_total'],
    }

    with open(out_dir / 'cs_baseline_summary.json', 'w') as f:
        json.dump(summary, f, indent=2)

    with open(out_dir / 'cs_baseline_results.json', 'w') as f:
        json.dump(results, f, indent=2)

    print(f'\n{"="*60}')
    print(f'COMPILER-SUGGESTION-ONLY BASELINE RESULTS')
    print(f'{"="*60}')
    print(f'Total attempts: {stats["total"]}')
    print(f'Fixed (0 errors after): {stats["fixed"]} ({strict_rate:.1f}%)')
    print(f'Improved (fewer errors): {stats["improved"]} ({improved_rate:.1f}%)')
    print(f'No suggestions available: {stats["no_suggestions"]}')
    print(f'Already compiles: {stats["already_compiles"]}')
    print(f'Errors: {stats["error"]}')
    print(f'Total suggestions found: {stats["suggestions_total"]} (Machine={stats["machine_total"]}, Maybe={stats["maybe_total"]})')
    print(f'Results saved to {out_dir}')

if __name__ == '__main__':
    main()
