/* Force-included into PCRE2's C sources for the WebAssembly build (see build.sh).
 * pcre2-sys turns on PCRE2's JIT for every target, but the JIT (sljit) has no WebAssembly
 * backend. Without SUPPORT_JIT, PCRE2 builds its interpreter only; the pcre2 crate's
 * jit_if_available(true) then falls back to it. */
#undef SUPPORT_JIT
