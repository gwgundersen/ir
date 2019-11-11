import base64
import ir
import pytest


@pytest.mark.parametrize("mode", ["tempfile", "memory"])
@pytest.mark.parametrize("format", ["text", "base64"])
def test_capture_echo(mode, format):
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


