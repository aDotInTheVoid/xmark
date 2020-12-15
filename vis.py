#!/usr/bin/env python3

import json

with open("target/doc/xmark.json") as f:
    raw = json.load(f)

print(raw["items"])
