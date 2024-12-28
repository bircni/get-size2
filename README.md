# get-size2

> This repo is a fork of get-size, as it is not maintained anymore. The original repo can be found [here](https://github.com/DKerp/get-size)

This repo contains two crates: `get-size2` and `get-size-derive2`.

## get-size2

Determine the size in bytes an object occupies inside RAM.

## get-size-derive2

The derive macro will provide a custom implementation of the [`get_heap_size`] method, which will simply call [`get_heap_size`] on all contained values and add the values up. This implies that by default all values contained in the struct or enum most implement the [`GetSize`] trait themselves.
