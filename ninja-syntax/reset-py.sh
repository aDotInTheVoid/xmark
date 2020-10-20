#!/bin/bash
set -euo pipefail

rm ninja_syntax*.py
curl -O https://raw.githubusercontent.com/ninja-build/ninja/master/misc/ninja_syntax.py
curl -O https://raw.githubusercontent.com/ninja-build/ninja/master/misc/ninja_syntax_test.py
