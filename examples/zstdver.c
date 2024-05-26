#include "zstdwrap.h"
#include <stdio.h>
#include <stdlib.h>

int
main (void)
{
#if defined(ZSTDWRAP_ENABLE_DLOPEN) && ZSTDWRAP_ENABLE_DLOPEN && \
  !defined(ZSTDWRAP_SONAME)
  const char *soname = getenv ("ZSTDWRAP_SONAME");
  if (!soname)
    abort();

  zstdwrap_ensure_library (soname, RTLD_LAZY | RTLD_LOCAL);
#endif

  unsigned v1 = ZSTDWRAP_FUNC(ZSTD_versionNumber)();
  printf ("ZSTD_versionNumber: %u\n", v1);
  const char *v2 = ZSTDWRAP_FUNC(ZSTD_versionString)();
  printf ("ZSTD_versionString: %s\n", v2);
  return 0;
}
