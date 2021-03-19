# Shard Affinity

This is a Proof of Concept project. See [explanation](https://github.com/waynexia/shard-affinity/blob/main/doc/explanation_zh-cn.md) for more detail.

# Structure

## src/
Contains test code of three loads.

## cache/
A hashmap-like memory cache.

## load/
Implementations of thee loads: threading, local-set and affinity.

## runtime/
Naive FIFO future driver, with each thread pinned to one core. Modified from [juliex](https://github.com/withoutboats/juliex).