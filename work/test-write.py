#!/usr/bin/env python

import random
import sys
import time

for c in range(32, 127):
    (sys.stdout if random.random() > 0.3 else sys.stderr).write(
        (1 << random.randint(0, 5)) * chr(c))
    time.sleep(random.random() * 0.1)

