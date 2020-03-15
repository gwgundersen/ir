import json
import os
from   pathlib import Path
import subprocess
import sys
import tempfile

#-------------------------------------------------------------------------------

IR_EXE = Path(__file__).parents[2] / "target/debug/ir"
TEST_EXE = Path(__file__).parent / "test.py"


class Errors(Exception):

    def __init__(self, errors):
        super().__init__("\n".join(errors))
        self.errors = tuple(errors)



def run(specs):
    specs = list(specs)
    with tempfile.NamedTemporaryFile(mode="w+") as tmp_file:
        json.dump({"procs": specs}, tmp_file)
        tmp_file.flush()
        res = subprocess.run(
            [str(IR_EXE), tmp_file.name],
            stdout=subprocess.PIPE,
            env={**os.environ, "RUST_BACKTRACE": "1"},
        )
    res = json.loads(res.stdout)
    json.dump(res, sys.stderr, indent=2)

    if len(res["errors"]) != 0:
        raise Errors(res["errors"])

    return res["procs"]


def run1(spec):
    # Return results for the single process only.
    proc, = run([spec])
    return proc


