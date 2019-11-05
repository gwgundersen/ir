#!/usr/bin/env python

import random
import sys
import time

for _ in range(100):
    (sys.stdout if random.random() > 0.3 else sys.stderr).write(
        (1 << random.randint(0, 4)) * chr(random.randint(32, 127)))
    time.sleep(random.random() * 0.1)

