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
