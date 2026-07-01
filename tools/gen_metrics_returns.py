#!/usr/bin/env python3
"""One-shot generator for the committed metrics-test input fixture.

Draws 252 pseudo-random daily returns from a fixed seed so the file is
reproducible, and writes them to `tests/data/metrics_returns.csv` — the
same file `gen_metrics_fixtures.py` and `tests/metrics_validation.rs`
both consume. Rerun only when the fixture needs to be replaced (in which
case regenerate the empyrical reference alongside it).
"""

import csv
import os

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "tests", "data", "metrics_returns.csv")

N = 252
SEED = 20260701
DAILY_MEAN = 0.0006  # ~15% annualized
DAILY_VOL = 0.015  # ~24% annualized


def main() -> None:
    rng = np.random.default_rng(SEED)
    returns = rng.normal(loc=DAILY_MEAN, scale=DAILY_VOL, size=N)
    with open(OUT, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["index", "return"])
        for i, r in enumerate(returns):
            w.writerow([i, repr(float(r))])
    print(f"wrote {OUT} ({N} returns, seed={SEED})")


if __name__ == "__main__":
    main()
