"""Convenience runner: python -m tests [pytest arguments]."""

from __future__ import annotations

import sys

import pytest


raise SystemExit(pytest.main(sys.argv[1:]))
