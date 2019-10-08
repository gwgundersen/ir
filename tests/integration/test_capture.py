import ir
import pytest


@pytest.mark.parametrize("mode", ["tempfile"])
def test_capture_echo(mode):
    res = ir.run({
        "argv": ["/bin/echo", "Hello, world.", "How are you?"],
        "fds": [
            ["stdout", {"capture": {"mode": mode}}],
        ]
    })

    assert res["status"] == 0
    assert res["fds"]["stdout"]["text"] == "Hello, world. How are you?\n"


