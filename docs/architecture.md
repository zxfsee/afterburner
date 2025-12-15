# Architecture Overview

This system models a minimal ML production loop:

Training -> Artifact -> Inference

## High-level flow

[ Training Binary ]
        |
        v
[ artifacts/inference/model.mpk ]
        |
        v
[ Inference Binary ]

Training produces artifacts.
Inference consumes artifacts.
No runtime coupling exists between training and inference.

