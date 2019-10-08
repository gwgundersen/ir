import ir
1

def test_stdout_stderr(tmp_path):
    stdout_path = tmp_path / "stdout"
    stderr_path = tmp_path / "stderr"
    res = ir.run({
        "argv": [str(ir.TEST_EXE), "--exit", "42"],
        "fds": {
            "1": {"file": {"path": str(stdout_path)}},
            "2": {"file": {"path": str(stderr_path)}},
        }
    })

    assert res["status"] == 42 << 8
    assert res["exit_code"] == 42
    assert res["signum"] is None
    assert res["core_dump"] is False

    assert stdout_path.read_text() == (
        "message 0 to stdout\n"
        "message 2 to stdout\n"
    )
    assert stderr_path.read_text() == (
        "message 1 to stderr\n"
    )


def test_stdout_stderr_merge(tmp_path):
    stderr_path = tmp_path / "stderr"
    res = ir.run({
        "argv": [str(ir.TEST_EXE), "--exit", "42"],
        "fds": {
            "stderr": {"file": {"path": str(stderr_path)}},
            "stdout": {"dup": {"fd": 2}},
        }
    })

    assert res["status"] == 42 << 8
    assert res["exit_code"] == 42
    assert res["signum"] is None
    assert res["core_dump"] is False

    assert stderr_path.read_text() == (
        "message 0 to stdout\n"
        "message 1 to stderr\n"
        "message 2 to stdout\n"
    )


