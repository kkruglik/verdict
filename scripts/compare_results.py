"""
Compares CLI JSON output against expected.json for a fixture folder.
Usage: python compare_results.py <fixture_dir> <actual_json>
"""
import json
import sys

fixture_dir = sys.argv[1]
actual_path = sys.argv[2]

with open(f"{fixture_dir}/expected.json") as f:
    expected = json.load(f)

with open(actual_path) as f:
    actual_full = json.load(f)

def normalize(c):
    return "is_in" if c.startswith("in_set") else c.split("(")[0]

actual = {(r["column"], normalize(r["constraint"])): r["passed"] for r in actual_full["results"]}

ok = True
for e in expected:
    key = (e["column"], e["constraint"])
    actual_passed = actual.get(key)
    if actual_passed is None:
        print(f"MISSING  {e['column']} / {e['constraint']}")
        ok = False
    elif actual_passed != e["passed"]:
        status = "WRONG  "
        exp = "pass" if e["passed"] else "fail"
        got = "pass" if actual_passed else "fail"
        print(f"{status}  {e['column']} / {e['constraint']}: expected={exp}, got={got}")
        ok = False
    else:
        exp = "pass" if e["passed"] else "fail"
        print(f"OK       {e['column']} / {e['constraint']}: {exp}")

print()
print("ALL OK" if ok else "FAILURES FOUND")
sys.exit(0 if ok else 1)
