import pytest

import ir

#-------------------------------------------------------------------------------

def test_bad_exe():
    """
    Tests error reporting on bad executable.
    """
    try:
        ir.run1({"argv": ["/usr/bin/bogus"],})
    except ir.Errors as exc:
        assert any( "No such file or directory" in e for e in exc.errors )


