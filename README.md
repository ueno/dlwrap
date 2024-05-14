# dlwrap

![Crates.io Version](https://img.shields.io/crates/v/dlwrap?link=https%3A%2F%2Fcrates.io%2Fcrates%2Fdlwrap) ![docs.rs](https://img.shields.io/docsrs/dlwrap?link=https%3A%2F%2Fdocs.rs%2Fdlwrap%2Flatest%2Fdlwrap%2F)

When creating an application that supports multiple backends (for
[compression][use-case-compression],
[cryptography][use-case-cryptography], etc.), it is sometimes
undesirable to link all supported libraries to the application at
once.

This can be solved by deferring loading of a library with `dlopen`
until the first time a function from the library is called. Such
mechanism is typically implemented through wrappers around the library
functions, though defining the wrappers is cumbersome and error-prone.

`dlwrap` makes it easily for an application to implement such
mechanism.

## Usage

Let's consider a hypothetical application which calls
`ZSTD_versionNumber` and `ZSTD_versionString` to retrieve the run-time
version of the ZSTD library.

First create a source file `zstdver.c` with the following:

```c
#include "zstdwrap.h"
#include <stdio.h>

int
main (void)
{
  unsigned v1 = ZSTDWRAP_FUNC(ZSTD_versionNumber)();
  printf ("ZSTD_versionNumber: %u\n", v1);
  const char *v2 = ZSTDWRAP_FUNC(ZSTD_versionString)();
  printf ("ZSTD_versionString: %s\n", v2);
  return 0;
}
```

A couple of things to note:

- Instead of the standard `<zstd.h>` header, `"zstdwrap.h"` is included
- Function symbols are wrapped with the `ZSTDWRAP_FUNC` macro

Now proceed to generate helper files:

```console
$ cargo install dlwrap
$ dlwrap --input /usr/include/zstd.h \
         --output-dir out \
         --clang-resource-dir "$(clang -print-resource-dir)" \
         --loader-basename zstdwrap \
         --symbol-regex "^ZSTD_(versionNumber|versionString)$" \
         --prefix zstdwrap \
         --include "<zstd.h>"
```

This command will create 3 files under `out/`: `zstdwrap.c`,
`zstdwrap.h`, and `zstdwrapfuncs.h`.

At this point the application can be compiled with:

```console
$ gcc -pthread -I./out \
      -DZSTDWRAP_ENABLE_DLOPEN=1 \
      -DZSTDWRAP_SONAME='"libzstd.so.1"' \
      -DZSTDWRAP_ENABLE_PTHREAD=1 \
      -o zstdver examples/zstdver.c out/zstdwrap.c
```

The generated code provides a couple of configuration macros:

- `<LIBRARY_PREFIX>_ENABLE_DLOPEN` controls whether to enable this
  `dlopen` mechanism at all. If it is undefined or 0, the application
  needs to be linked to the required library at build time (see
  below). This is useful when conditionalizing builds based on
  platforms where `dlopen` is supported or not.

- `<LIBRARY_PREFIX>_SONAME` specifies the first argument to
  `dlopen`. If it is undefined, no library is loaded automatically and
  the application needs to call `<library_prefix>_ensure_library`
  function, which takes the library SONAME as the first argument. This
  is useful when the application determines the actual library at run
  time.

- `<LIBRARY_PREFIX>_ENABLE_PTHREAD` controls whether the automatic
  library loading is supposed to be thread safe; in that case
  `pthread_once` is used to protect library loading and symbol
  resolution.

When inspecting the `zstdver` executable with `ldd`, you will see
`libzstd.so.1` is not linked, but it works as if it is:

```console
$ ldd zstdver
        linux-vdso.so.1 (0x00007ffc705bf000)
        libc.so.6 => /lib64/libc.so.6 (0x00007f8a1199f000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f8a11ba3000)

$ ./zstdver
ZSTD_versionNumber: 10506
ZSTD_versionString: 1.5.6

$ ltrace -e dlopen ./zstdver
zstdver->dlopen("libzstd.so.1", 1)               = 0x13152c0
ZSTD_versionNumber: 10506
ZSTD_versionString: 1.5.6
+++ exited (status 0) +++
```

Without `ZSTDWRAP_ENABLE_DLOPEN` defined, the same application code
can be compiled with the standard linkage (i.e., `libzstd` is linked
at build time):

```console
$ gcc -I./out \
      -o zstdver examples/zstdver.c out/zstdwrap.c \
      -lzstd

$ ldd ./zstdver
        linux-vdso.so.1 (0x00007ffcd43e4000)
        libzstd.so.1 => /lib64/libzstd.so.1 (0x00007f7323269000)
        libc.so.6 => /lib64/libc.so.6 (0x00007f7323087000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f732334a000)

$ ltrace -e dlopen ./zstdver
ZSTD_versionNumber: 10506
ZSTD_versionString: 1.5.6
+++ exited (status 0) +++
```

## License

Apache-2.0

[use-case-compression]: https://gitlab.com/gnutls/gnutls/-/issues/1424
[use-case-cryptography]: https://github.com/open-quantum-safe/liboqs/pull/1603
