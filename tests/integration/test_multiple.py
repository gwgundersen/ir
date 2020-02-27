import ir


def test_multiple():
    res = ir.run(*(
        {
            "argv": ["/bin/echo", f"This is process #{i}."],
            "fds": [
                ["stdout", {"capture": {"mode": "memory"}}],
            ],
        }
        for i in range(8)
    ))

    assert len(res) == 8
    for i, proc in enumerate(res):
        assert proc["status"] == 0
        assert proc["fds"]["stdout"]["text"] == f"This is process #{i}.\n"


