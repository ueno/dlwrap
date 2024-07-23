/*
 * Copying and distribution of this file, with or without modification,
 * are permitted in any medium without royalty provided the copyright
 * notice and this notice are preserved.  This file is offered as-is,
 * without any warranty.
 */

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "array.h"

#if defined(ARRAY_ENABLE_DLOPEN) && ARRAY_ENABLE_DLOPEN

#include <assert.h>
#include <dlfcn.h>
#include <errno.h>
#include <stdlib.h>

/* If ARRAY_SONAME is defined, dlopen handle can be automatically
 * set; otherwise, the caller needs to call
 * array_ensure_library with soname determined at run time.
 */
#ifdef ARRAY_SONAME

static void
ensure_library (void)
{
  if (array_ensure_library (ARRAY_SONAME, RTLD_LAZY | RTLD_LOCAL) < 0)
    abort ();
}

#if defined(ARRAY_ENABLE_PTHREAD) && ARRAY_ENABLE_PTHREAD
#include <pthread.h>

static pthread_once_t dlopen_once = PTHREAD_ONCE_INIT;

#define ENSURE_LIBRARY pthread_once(&dlopen_once, ensure_library)

#else /* ARRAY_ENABLE_PTHREAD */

#define ENSURE_LIBRARY do {	    \
    if (!array_dlhandle) \
      ensure_library();		    \
  } while (0)

#endif /* !ARRAY_ENABLE_PTHREAD */

#else /* ARRAY_SONAME */

#define ENSURE_LIBRARY do {} while (0)

#endif /* !ARRAY_SONAME */

static void *array_dlhandle;

/* Define redirection symbols */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#if (2 <= __GNUC__ || (4 <= __clang_major__))
#define FUNC(ret, name, args, cargs)			\
  static __typeof__(name)(*array_sym_##name);
#else
#define FUNC(ret, name, args, cargs)		\
  static ret(*array_sym_##name)args;
#endif
#define VOID_FUNC FUNC
#include "arrayfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop

/* Define redirection wrapper functions */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#define FUNC(ret, name, args, cargs)        \
ret array_func_##name args           \
{					    \
  ENSURE_LIBRARY;			    \
  assert (array_sym_##name);	    \
  return array_sym_##name cargs;	    \
}
#define VOID_FUNC(ret, name, args, cargs)   \
ret array_func_##name args           \
{					    \
  ENSURE_LIBRARY;			    \
  assert (array_sym_##name);	    \
  array_sym_##name cargs;		    \
}
#include "arrayfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop

static int
ensure_symbol (const char *name, void **symp)
{
  if (!*symp)
    {
      void *sym = dlsym (array_dlhandle, name);
      if (!sym)
	return -errno;
      *symp = sym;
    }
  return 0;
}

int
array_ensure_library (const char *soname, int flags)
{
  int err;

  if (!array_dlhandle)
    {
      array_dlhandle = dlopen (soname, flags);
      if (!array_dlhandle)
	return -errno;
    }

#define ENSURE_SYMBOL(name)					\
  ensure_symbol(#name, (void **)&array_sym_##name)

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#define FUNC(ret, name, args, cargs)	\
  err = ENSURE_SYMBOL(name);		\
  if (err < 0)				\
    return err;
#define VOID_FUNC FUNC
#include "arrayfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop

#undef ENSURE_SYMBOL
  return 0;
}

void
array_unload_library (void)
{
  if (array_dlhandle)
    {
      dlclose (array_dlhandle);
      array_dlhandle = NULL;
    }

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#define FUNC(ret, name, args, cargs)		\
  array_sym_##name = NULL;
#define VOID_FUNC FUNC
#include "arrayfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop
}

#else /* ARRAY_ENABLE_DLOPEN */

int
array_ensure_library (const char *soname, int flags)
{
  (void) soname;
  (void) flags;
  return 0;
}

void
array_unload_library (void)
{
}

#endif /* !ARRAY_ENABLE_DLOPEN */
