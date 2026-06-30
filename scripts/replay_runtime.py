#!/usr/bin/env python3
"""
Replay all compile_success RUG-generated tests on clean crate snapshots,
run cargo test, and report runtime pass/fail distribution.
Focus: failure rate and failure categories among compiled tests.
"""
import json, os, sys, shutil, subprocess, tempfile, re
from pathlib import Path
from collections import defaultdict

EVAL = Path(os.environ.get('EVAL_DIR', '.'))
CRATES = EVAL / 'crates'
RUG_RUNS = EVAL / 'rug_runs'

# ---- config ----
TARGET_RUNS = [
    'humantime_gemini-2.5-flash-nothinking_20251109_134926',
    'semver_gemini-2.5-flash-nothinking_20251127_013324',
    'itoa_gemini-2.5-flash-nothinking_20251127_010109',
    'log_gemini-2.5-flash-nothinking_20251127_025911',
    'ryu_gemini-2.5-flash-nothinking_20251127_013142',
    'rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808',
    'rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158',
]

def crate_name_from_run(run_name):
    """Extract crate name: 'humantime_gemini-...' -> 'humantime'"""
    parts = run_name.split('_')
    # Find where the model name starts (contains digits or known model patterns)
    for i, p in enumerate(parts):
        if any(c.isdigit() for c in p) and i > 0:
            return '_'.join(parts[:i])
    return parts[0]

def find_source_file(crate_path, fn_path):
    """Map function path like 'duration::parse_duration' to source file."""
    module_parts = fn_path.split('::')
    # The first segment is the module; look for <module>.rs or <module>/mod.rs
    # For impl blocks like '<u64 as duration::OverflowOp>::mul', the module is the trait's module
    module = module_parts[0]
    # Clean generic/impl syntax
    module = re.sub(r'<.*?>', '', module).strip()
    if not module:
        # It's an impl block - find the module from the path
        for part in module_parts:
            clean = re.sub(r'<.*?>', '', part).strip()
            if clean and not clean[0].isupper():
                module = clean
                break

    src_dir = crate_path / 'src'
    candidates = [
        src_dir / f'{module}.rs',
        src_dir / module / 'mod.rs',
        src_dir / 'lib.rs',  # fallback
    ]
    for c in candidates:
        if c.exists():
            return c
    return src_dir / 'lib.rs'

def inject_tests(crate_path, test_modules_by_file):
    """Append test modules to their respective source files."""
    for file_path, modules in test_modules_by_file.items():
        with open(file_path, 'a') as f:
            f.write('\n')
            for code in modules:
                f.write(code)
                f.write('\n')

def parse_test_output(stdout, stderr):
    """Parse cargo test output to extract pass/fail counts and failure details."""
    result = {
        'total': 0, 'passed': 0, 'failed': 0, 'ignored': 0,
        'failures': [], 'test_result_line': '',
    }

    combined = stdout + '\n' + stderr

    # Find the test result summary line
    for line in combined.split('\n'):
        if 'test result:' in line:
            result['test_result_line'] = line.strip()
            # Parse: "test result: FAILED. 5 passed; 3 failed; 0 ignored"
            m = re.search(r'(\d+)\s+passed', line)
            if m: result['passed'] = int(m.group(1))
            m = re.search(r'(\d+)\s+failed', line)
            if m: result['failed'] = int(m.group(1))
            m = re.search(r'(\d+)\s+ignored', line)
            if m: result['ignored'] = int(m.group(1))
            result['total'] = result['passed'] + result['failed'] + result['ignored']

    # Extract individual failure details
    in_failures = False
    current_failure = None
    for line in combined.split('\n'):
        if line.startswith('failures:'):
            in_failures = True
            continue
        if in_failures:
            if line.startswith('test ') and '... FAILED' in line:
                name = line.split(' ...')[0].replace('test ', '').strip()
                current_failure = {'test': name, 'reason': ''}
                result['failures'].append(current_failure)
            elif current_failure and line.strip() and not line.startswith('----'):
                # Failure detail
                pass
            elif current_failure and ('panicked at' in line or 'assertion' in line):
                current_failure['reason'] = line.strip()[:200]

    return result

def categorize_failure(failure):
    """Categorize a test failure."""
    reason = failure.get('reason', '')
    test_name = failure.get('test', '')
    if 'assertion' in reason.lower() or 'assert' in reason.lower():
        return 'assertion_error'
    if 'panicked' in reason.lower():
        if 'unwrap' in reason or 'expect' in reason:
            return 'unwrap_panic'
        if 'index out of bounds' in reason:
            return 'index_oob'
        if 'overflow' in reason.lower():
            return 'arithmetic_overflow'
        return 'other_panic'
    if 'did not panic' in reason:
        return 'should_panic_mismatch'
    return 'unknown'

def main():
    results = {}

    for run_name in TARGET_RUNS:
        run_dir = RUG_RUNS / run_name
        log_path = run_dir / 'detailed_log.json'
        if not log_path.exists():
            print(f'SKIP {run_name}: no detailed_log.json')
            continue

        crate_name = crate_name_from_run(run_name)
        clean_crate = CRATES / crate_name
        if not clean_crate.exists():
            print(f'SKIP {run_name}: no clean crate at {clean_crate}')
            continue

        print(f'\n{"="*60}')
        print(f'Processing: {run_name}')
        print(f'  Crate: {crate_name}')

        with open(log_path) as f:
            data = json.load(f)

        # Collect compile_success test modules, grouped by file
        test_modules_by_file = defaultdict(list)
        total_tests = 0
        compile_success_count = 0

        for fn_key, fn_data in data.items():
            tg = fn_data.get('test_generation', [])
            if not isinstance(tg, list):
                continue
            for attempt in tg:
                if attempt.get('compile_success') or attempt.get('success'):
                    compile_success_count += 1
                    code = attempt.get('injected_code', '')
                    if not code.rstrip().endswith('}'):
                        continue  # skip truncated

                    # Count #[test] functions in this module
                    test_count = len(re.findall(r'#\[test\]', code))
                    total_tests += test_count

                    src_file = find_source_file(clean_crate, fn_key)
                    test_modules_by_file[src_file].append(code)

        print(f'  Compile-success attempts: {compile_success_count}')
        print(f'  Total #[test] functions: {total_tests}')
        print(f'  Source files to modify: {len(test_modules_by_file)}')

        # Copy crate to temp
        tmp = Path(tempfile.mkdtemp(prefix=f'rug_replay_{crate_name}_'))
        shutil.copytree(clean_crate, tmp, dirs_exist_ok=True, symlinks=False)

        # Remap paths to tmp
        tmp_test_modules = {}
        for src_file, modules in test_modules_by_file.items():
            rel = src_file.relative_to(clean_crate)
            tmp_file = tmp / rel
            tmp_test_modules[tmp_file] = modules

        # Inject tests
        inject_tests(tmp, tmp_test_modules)

        # Run cargo test
        print(f'  Running cargo test...')
        try:
            proc = subprocess.run(
                ['cargo', 'test'],
                cwd=tmp,
                capture_output=True, text=True,
                timeout=300,
            )
        except subprocess.TimeoutExpired:
            print(f'  TIMEOUT')
            shutil.rmtree(tmp, ignore_errors=True)
            results[run_name] = {'status': 'timeout', 'crate': crate_name}
            continue

        stdout = proc.stdout
        stderr = proc.stderr

        # Check for compile errors
        compile_failed = 'error[' in stdout + stderr or 'error:' in stdout + stderr

        if compile_failed:
            # Check if tests still ran
            if 'test result:' in stdout:
                pass  # some errors might be warnings
            else:
                print(f'  COMPILE FAILED (test injection caused new errors)')
                # Extract first few errors
                error_lines = [l for l in (stdout + stderr).split('\n') if 'error[' in l or 'error:' in l]
                for el in error_lines[:5]:
                    print(f'    {el[:150]}')

        # Parse test results
        parsed = parse_test_output(stdout, stderr)

        # Categorize failures
        failure_cats = defaultdict(list)
        for f in parsed['failures']:
            cat = categorize_failure(f)
            failure_cats[cat].append(f['test'])

        print(f'  Tests: {parsed["total"]} total, {parsed["passed"]} passed, {parsed["failed"]} failed')
        if parsed['failed'] > 0:
            print(f'  Failure categories:')
            for cat, tests in sorted(failure_cats.items()):
                print(f'    {cat}: {len(tests)} ({len(tests)/parsed["failed"]*100:.0f}%)')
                for t in tests[:3]:
                    print(f'      - {t}')

        results[run_name] = {
            'crate': crate_name,
            'status': 'ok',
            'compile_success_attempts': compile_success_count,
            'injected_test_functions': total_tests,
            'compile_failed': compile_failed,
            **parsed,
            'failure_categories': {k: len(v) for k, v in failure_cats.items()},
        }

        # Cleanup
        shutil.rmtree(tmp, ignore_errors=True)

    # ---- Aggregate ----
    print(f'\n{"="*60}')
    print('AGGREGATE RESULTS')
    print(f'{"="*60}')

    grand_total = 0
    grand_passed = 0
    grand_failed = 0
    all_cats = defaultdict(int)

    for run_name, r in sorted(results.items()):
        if r.get('status') != 'ok':
            print(f'  {run_name[:50]}: {r["status"]}')
            continue
        t, p, f = r['total'], r['passed'], r['failed']
        rate = f'{p/t*100:.1f}%' if t > 0 else 'N/A'
        grand_total += t
        grand_passed += p
        grand_failed += f
        for cat, count in r.get('failure_categories', {}).items():
            all_cats[cat] += count
        print(f'  {r["crate"]:<20s}  tests={t:>4d}  pass={p:>4d}  fail={f:>4d}  pass_rate={rate}')

    overall_rate = f'{grand_passed/grand_total*100:.1f}%' if grand_total > 0 else 'N/A'
    print(f'\n  {"TOTAL":<20s}  tests={grand_total:>4d}  pass={grand_passed:>4d}  fail={grand_failed:>4d}  pass_rate={overall_rate}')

    if grand_failed > 0:
        print(f'\n  Failure distribution:')
        for cat, count in sorted(all_cats.items(), key=lambda x: -x[1]):
            print(f'    {cat}: {count} ({count/grand_failed*100:.0f}%)')

    # Save results
    out_path = EVAL / 'local_analysis_output' / 'runtime_replay_results.json'
    with open(out_path, 'w') as f:
        json.dump({
            'runs': {k: {kk: vv for kk, vv in v.items() if kk != 'failures'} for k, v in results.items()},
            'aggregate': {
                'total_tests': grand_total,
                'passed': grand_passed,
                'failed': grand_failed,
                'pass_rate': overall_rate,
                'failure_categories': dict(all_cats),
            }
        }, f, indent=2)
    print(f'\nResults saved to {out_path}')

if __name__ == '__main__':
    main()
