#!/bin/bash -e
rustc --version || (echo "rustc not found - please install rustc and cargo" && exit 1)
python3.10 -m venv .venv
source .venv/bin/activate
pip-sync || pip install pip-tools==6.13.0 && pip-sync