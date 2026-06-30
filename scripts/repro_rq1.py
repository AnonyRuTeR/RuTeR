#!/usr/bin/env python3
"""
RQ1 Reproduction v2: deduplicate test module names, compile per-run,
aggregate compile error-code distributions.
"""
import json, os, sys, re, shutil, subprocess, tempfile, hashlib
from pathlib import Path
from collections import defaultdict

EVAL = Path(os.environ.get('EVAL_DIR', '.'))
CRATES = EVAL / 'crates'
RUG_RUNS = EVAL / 'rug_runs'
OUTPUT = EVAL / 'local_analysis_output' / 'rq1_repro'

RUNS = sorted([
    d.name for d in RUG_RUNS.iterdir()
    if d.is_dir() and (d / 'detailed_log.json').exists()
])

def crate_from_run(run_name):
    model_patterns = ['gemini', 'gpt', 'claude', 'deepseek']
    parts = run_name.split('_')
    for i, p in enumerate(parts):
        for mp in model_patterns:
            if p.startswith(mp):
                return '_'.join(parts[:i]), '_'.join(parts[i:]).rsplit('_', 2)[0]
    return run_name, 'unknown'

def find_target_file(crate_path, fn_key):
    module = re.sub(r'<.*?>', '', fn_key.split('::')[0]).strip()
    if not module:
        for part in fn_key.split('::'):
            clean = re.sub(r'<.*?>', '', part).strip()
            if clean and not clean[0].isupper():
                module = clean; break
    src = crate_path / 'src'
    for c in [src / f'{module}.rs', src / module / 'mod.rs', src / 'lib.rs']:
        if c.exists():
            return c
    return src / 'lib.rs'

def dedup_module_names(code, fn_key):
    """Rename test modules to avoid collisions when injecting all tests at once."""
    short_hash = hashlib.md5(fn_key.encode()).hexdigest()[:6]
    def replacer(m):
        mod_name = m.group(1)
        new_name = f'{mod_name}_{short_hash}'
        return f'mod {new_name}'
    return re.sub(r'mod\s+(tests_rug_\d+)', replacer, code)

def process_run(run_name):
    run_dir = RUG_RUNS / run_name
    log_path = run_dir / 'detailed_log.json'
    crate_name, model = crate_from_run(run_name)
    clean_crate = CRATES / crate_name

    result = {
        'run': run_name, 'crate': crate_name, 'model': model,
        'status': 'skipped', 'total_attempts': 0, 'injected_modules': 0,
        'compile_ok': False, 'errors': {},
    }

    if not clean_crate.exists():
        result['status'] = 'no_crate_snapshot'
        return result

    with open(log_path) as f:
        data = json.load(f)

    by_file = defaultdict(list)
    total_attempts = 0
    injected_modules = 0

    for fn_key, fn_data in data.items():
        tg = fn_data.get('test_generation', [])
        if not isinstance(tg, list): continue
        total_attempts += len(tg)
        # Pick best attempt per function: prefer compile_success=True, else last
        best = None
        for attempt in tg:
            best = attempt
            if attempt.get('compile_success'):
                break  # take the successful one
        if best is None: continue
        code = best.get('injected_code', '')
        if not code.strip(): continue
        if not code.rstrip().endswith('}'): continue
        code = dedup_module_names(code, fn_key)
        target = find_target_file(clean_crate, fn_key)
        by_file[target].append(code)
        injected_modules += 1

    result['total_attempts'] = total_attempts
    result['injected_modules'] = injected_modules

    if injected_modules == 0:
        result['status'] = 'no_valid_code'
        return result

    tmp = Path(tempfile.mkdtemp(prefix=f'rq1_{crate_name}_'))
    try:
        shutil.copytree(clean_crate, tmp, dirs_exist_ok=True)
    except Exception as e:
        shutil.rmtree(tmp, ignore_errors=True)
        result['status'] = f'copy_failed'
        return result

    for src_file, codes in by_file.items():
        rel = src_file.relative_to(clean_crate)
        target = tmp / rel
        with open(target, 'a') as f:
            f.write('\n')
            for code in codes:
                f.write(code + '\n')

    # Run cargo check --tests
    try:
        proc = subprocess.run(
            ['cargo', 'check', '--tests'],
            cwd=tmp, capture_output=True, text=True, timeout=300,
        )
    except subprocess.TimeoutExpired:
        shutil.rmtree(tmp, ignore_errors=True)
        result['status'] = 'timeout'
        return result

    stdout, stderr = proc.stdout, proc.stderr
    combined = stdout + stderr

    # Parse error codes
    errors = defaultdict(int)
    warning_count = 0
    for line in combined.split('\n'):
        # Match: error[E0433]
        for m in re.finditer(r'error\[(E\d+)\]', line):
            errors[m.group(1)] += 1

    compile_ok = proc.returncode == 0 and not errors

    # Check for compilation failures without error codes
    if not compile_ok and not errors:
        # Might have "error: could not compile" without specific codes
        if 'could not compile' in combined:
            errors['COMPILE_FAILED'] = 1

    shutil.rmtree(tmp, ignore_errors=True)

    result['status'] = 'ok'
    result['compile_ok'] = compile_ok
    result['errors'] = dict(errors)
    return result


def main():
    print(f'RQ1 Reproduction v2 (dedup module names): {len(RUNS)} runs\n')

    all_results = []
    all_errors = defaultdict(int)
    ok_runs = 0
    fail_runs = 0

    for i, run_name in enumerate(RUNS):
        crate, model = crate_from_run(run_name)
        print(f'[{i+1:>2}/{len(RUNS)}] {crate:<20s} {model[:30]:<30s} ...', end=' ', flush=True)
        r = process_run(run_name)
        all_results.append(r)

        if r['status'] != 'ok':
            print(f'SKIP ({r["status"]})')
            continue

        n_err = sum(r['errors'].values())
        if r['compile_ok']:
            ok_runs += 1
            print(f'COMPILE OK ({r["injected_modules"]} modules)')
        else:
            fail_runs += 1
            top = sorted(r['errors'].items(), key=lambda x: -x[1])[:4]
            print(f'{n_err} errors: {", ".join(f"{c}={n}" for c,n in top)}')

        for code, count in r['errors'].items():
            all_errors[code] += count

    # Summary
    print(f'\n{"="*70}')
    print('RQ1 REPRODUCTION SUMMARY')
    print(f'{"="*70}')
    print(f'Runs: {len(RUNS)} total, {ok_runs} compile-OK, {fail_runs} with errors')
    print(f'\nError code distribution (all runs combined, excluding E0428):')
    ranked = sorted(all_errors.items(), key=lambda x: -x[1])
    filtered = [(c, n) for c, n in ranked if c != 'E0428']
    for i, (code, count) in enumerate(filtered[:12]):
        bar = '█' * (count // 30)
        print(f'  {i+1:>2}. {code:<10s} {count:>6d}  {bar}')

    # Cross-crate (gemini fixed)
    print(f'\n{"="*70}')
    print('CROSS-CRATE (gemini-2.5-flash-nothinking)')
    print(f'{"="*70}')
    gem_runs = [r for r in all_results if 'gemini-2.5-flash-nothinking' in r['run']]
    for r in sorted(gem_runs, key=lambda x: x['crate']):
        errs = r.get('errors', {})
        top3 = sorted(((c,n) for c,n in errs.items() if c!='E0428'), key=lambda x:-x[1])[:3]
        print(f'  {r["crate"]:<20s}  mods={r.get("injected_modules",0):>4d}  top: {", ".join(f"{c}={n}" for c,n in top3) if top3 else "none"}')

    # Cross-model (humantime fixed)
    print(f'\n{"="*70}')
    print('CROSS-MODEL (humantime)')
    print(f'{"="*70}')
    hum_runs = [r for r in all_results if r['crate'] == 'humantime']
    for r in sorted(hum_runs, key=lambda x: x['model']):
        errs = r.get('errors', {})
        top3 = sorted(((c,n) for c,n in errs.items() if c!='E0428'), key=lambda x:-x[1])[:3]
        print(f'  {r["model"]:<35s}  mods={r.get("injected_modules",0):>4d}  top: {", ".join(f"{c}={n}" for c,n in top3) if top3 else "none"}')

    # Save
    os.makedirs(OUTPUT, exist_ok=True)
    summary = {
        'runs_processed': len(RUNS),
        'compile_ok_runs': ok_runs,
        'compile_error_runs': fail_runs,
        'error_distribution': dict(ranked),
        'per_run': all_results,
    }
    with open(OUTPUT / 'rq1_repro_v2_summary.json', 'w') as f:
        json.dump(summary, f, indent=2, default=lambda x: list(x) if isinstance(x, set) else str(x))
    print(f'\nSaved to {OUTPUT / "rq1_repro_v2_summary.json"}')

if __name__ == '__main__':
    main()
