import json
import os
from   pathlib import Path
import subprocess
import sys
import tempfile

#-------------------------------------------------------------------------------

IR_EXE = Path(__file__).parents[2] / "target/debug/ir"
TEST_EXE = Path(__file__).parent / "test.py"


def run(spec):
    with tempfile.NamedTemporaryFile(mode="w+") as tmp_file:
        json.dump(spec, tmp_file)
        tmp_file.flush()
        res = subprocess.run(
            [str(IR_EXE), tmp_file.name],
            stdout=subprocess.PIPE,
            env={**os.environ, "RUST_BACKTRACE": "1"},
        )
        assert res.returncode == 0, f"ir exited {res.returncode}"
    res = json.loads(res.stdout)
    json.dump(res, sys.stderr, indent=2)
    # Return results for the single process only.
    return res["procs"][0]


