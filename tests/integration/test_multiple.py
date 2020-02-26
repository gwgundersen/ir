import ir


def test_multiple():
    res = ir.run(*(
        {
            "argv": ["/bin/echo", f"This is process #{i}."],
            "fds": [
                ["stdout", {"capture": {"mode": "memory"}}],
            ],
        }
        for i in range(3)
    ))
    print(res)
    assert False

