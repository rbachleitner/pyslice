#!/bin/bash -e
source .venv/bin/activate
maturin develop && time python test_stl.py Sphere.stl 1