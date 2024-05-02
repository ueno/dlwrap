# dlwrap

When creating an application that supports multiple backends (for
compression, config file formats, cryptography, etc.), it is sometimes
undesirable to link all supported libraries to the application, as it
would add those libraries as a dependency of the application.

This can be solved by deferring loading of the library with `dlopen`
until the first time a function from the library is called. This
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
$ cargo run -- --input /usr/include/zstd.h \
         --output out \
         --loader zstdwrap \
         --function-regex "^ZSTD_(versionNumber|versionString)$" \
         --library-prefix zstdwrap \
         --symbol-prefix zstdwrap_sym \
         --function-prefix zstdwrap_func \
         --soname ZSTDWRAP_SONAME \
         --wrapper ZSTDWRAP_FUNC \
         --header=zstd.h
```

This command will create 3 files under `out/`: `zstdwrap.c`,
`zstdwrap.h`, and `zstdwrapfuncs.h`.

At this point the application can be compiled with:

```console
$ gcc -pthread -I./out \
      -DZSTDWRAP_ENABLE_PTHREAD=1 \
      -DZSTDWRAP_ENABLE_DLOPEN=1 \
      -DZSTDWRAP_SONAME='"libzstd.so.1"' \
      -o zstdver examples/zstdver.c out/zstdwrap.c
```

`ZSTDWRAP_ENABLE_PTHREAD` controls whether the application is suppsed
to be thread safe; in that case `pthread_once` is used to protect
library loading and symbol resolution.

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

With `ZSTDWRAP_ENABLE_DLOPEN=0`, the same application code can be
compiled with the standard linkage (i.e., `libzstd` is linked at build
time):

```console
$ gcc -I./out \
      -DZSTDWRAP_ENABLE_DLOPEN=0 \
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
