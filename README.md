## The even better, ridiculously amazing `cffi-explore`


This project is a `rust` wrapper around a hypothetical `c++` library
(see [libdummy](libdummy/README.md) ) that exposes a `C` abi.
The goal of this repo is to document/explore how to write such a `rust` wrapper 
from the perspective of someone not only new to `rust`, but new to writing
non garbage collected code in general.



One very **important** caveat is that my `c/c++` knowledge is **very** limited.
I've lived most of my life in the `Java/JS/Python` land of GC goodness.
This is an attempt to get my feet wet on the mystical world where ___memory management___
is ... manual :scream:



## Notes to `&self`

- [Passing dyn Trait through ffi](notes/fatptr_through_ffi.md)
- [Memory leaks](notes/free_mem.md)