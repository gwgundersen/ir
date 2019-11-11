import base64
import ir
from   pathlib import Path
import pytest

TEST_DIR = Path(__file__).parent

@pytest.mark.parametrize("mode", ["tempfile", "memory"])
@pytest.mark.parametrize("format", ["text", "base64"])
def test_echo(mode, format):
    """
    Basic test of capturing stdout.
    """
    res = ir.run({
        "argv": ["/bin/echo", "Hello, world.", "How are you?"],
        "fds": [
            [
                "stdout", {
                    "capture": {
                        "mode": mode,
                        "format": format,
                    }
                }
            ],
        ]
    })

    assert res["status"] == 0

    stdout = res["fds"]["stdout"]
    text = "Hello, world. How are you?\n"
    if mode == "text":
        assert stdout["text"] == text
    elif mode == "base64":
        assert stdout["encoding"] == "base64"
        assert stdout["text"] == base64.b64encode(text.encode())


@pytest.mark.parametrize("mode", ["tempfile", "memory"])
def test_interleaved(mode):
    """
    Test of interleaved stdout and stderr.
    """
    exe = TEST_DIR / "interleaved.py"
    assert exe.exists()

    res = ir.run({
        "argv": [
            str(exe),
        ],
        "fds": [
            [
                "stdout", {
                    "capture": {
                        "mode": mode,
                        "format": "base64",
                    }
                },
            ],
            [
                "stderr", {
                    "capture": {
                        "mode": mode,
                        "format": "base64",
                    }
                },
            ]
        ]
    })

    assert res["status"] == 0

    out = base64.standard_b64decode(res["fds"]["stdout"]["data"])
    err = base64.standard_b64decode(res["fds"]["stderr"]["data"])
    assert out == b"".join( bytes([i]) * i for i in range(256) if i % 3 != 0 )
    assert err == b"".join( bytes([i]) * i for i in range(256) if i % 3 == 0 )

    
