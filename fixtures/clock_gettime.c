#include <time.h>
#include "cgwrap.h"

int
main (void)
{
  struct timespec ts;

  CGWRAP_FUNC(clock_gettime) (CLOCK_MONOTONIC, &ts);

  return 0;
}
