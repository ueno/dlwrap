/*
 * Copying and distribution of this file, with or without modification,
 * are permitted in any medium without royalty provided the copyright
 * notice and this notice are preserved.  This file is offered as-is,
 * without any warranty.
 */

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "cgwrap.h"

#if defined(CGWRAP_ENABLE_DLOPEN) && CGWRAP_ENABLE_DLOPEN

#include <assert.h>
#include <dlfcn.h>
#include <errno.h>
#include <stdlib.h>

/* If CGWRAP_SONAME is defined, dlopen handle can be automatically
 * set; otherwise, the caller needs to call
 * cgwrap_ensure_library with soname determined at run time.
 */
#ifdef CGWRAP_SONAME

static void
ensure_library (void)
{
  if (cgwrap_ensure_library (CGWRAP_SONAME, RTLD_LAZY | RTLD_LOCAL) < 0)
    abort ();
}

#if defined(CGWRAP_ENABLE_PTHREAD) && CGWRAP_ENABLE_PTHREAD
#include <pthread.h>

static pthread_once_t dlopen_once = PTHREAD_ONCE_INIT;

#define ENSURE_LIBRARY pthread_once(&dlopen_once, ensure_library)

#else /* CGWRAP_ENABLE_PTHREAD */

#define ENSURE_LIBRARY do {	    \
    if (!cgwrap_dlhandle) \
      ensure_library();		    \
  } while (0)

#endif /* !CGWRAP_ENABLE_PTHREAD */

#else /* CGWRAP_SONAME */

#define ENSURE_LIBRARY do {} while (0)

#endif /* !CGWRAP_SONAME */

static void *cgwrap_dlhandle;

/* Define redirection symbols */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#if (2 <= __GNUC__ || (4 <= __clang_major__))
#define FUNC(ret, name, args, cargs)			\
  static __typeof__(name)(*cgwrap_sym_##name);
#else
#define FUNC(ret, name, args, cargs)		\
  static ret(*cgwrap_sym_##name)args;
#endif
#define VOID_FUNC FUNC
#include "cgwrapfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop

/* Define redirection wrapper functions */
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#define FUNC(ret, name, args, cargs)        \
ret cgwrap_func_##name args           \
{					    \
  ENSURE_LIBRARY;			    \
  assert (cgwrap_sym_##name);	    \
  return cgwrap_sym_##name cargs;	    \
}
#define VOID_FUNC(ret, name, args, cargs)   \
ret cgwrap_func_##name args           \
{					    \
  ENSURE_LIBRARY;			    \
  assert (cgwrap_sym_##name);	    \
  cgwrap_sym_##name cargs;		    \
}
#include "cgwrapfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop

static int
ensure_symbol (const char *name, void **symp)
{
  if (!*symp)
    {
      void *sym = dlsym (cgwrap_dlhandle, name);
      if (!sym)
	return -EINVAL;
      *symp = sym;
    }
  return 0;
}

int
cgwrap_ensure_library (const char *soname, int flags)
{
  int err;

  if (!cgwrap_dlhandle)
    {
      cgwrap_dlhandle = dlopen (soname, flags);
      if (!cgwrap_dlhandle)
	return -EINVAL;
    }

#define ENSURE_SYMBOL(name)					\
  ensure_symbol(#name, (void **)&cgwrap_sym_##name)

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#define FUNC(ret, name, args, cargs)		\
  err = ENSURE_SYMBOL(name);			\
  if (err < 0)					\
    {						\
      cgwrap_dlhandle = NULL;		\
      return err;				\
    }
#define VOID_FUNC FUNC
#include "cgwrapfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop

#undef ENSURE_SYMBOL
  return 0;
}

void
cgwrap_unload_library (void)
{
  if (cgwrap_dlhandle)
    {
      dlclose (cgwrap_dlhandle);
      cgwrap_dlhandle = NULL;
    }

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-macros"

#define FUNC(ret, name, args, cargs)		\
  cgwrap_sym_##name = NULL;
#define VOID_FUNC FUNC
#include "cgwrapfuncs.h"
#undef VOID_FUNC
#undef FUNC

#pragma GCC diagnostic pop
}

unsigned
cgwrap_is_usable (void)
{
  return cgwrap_dlhandle != NULL;
}

#else /* CGWRAP_ENABLE_DLOPEN */

int
cgwrap_ensure_library (const char *soname, int flags)
{
  (void) soname;
  (void) flags;
  return 0;
}

void
cgwrap_unload_library (void)
{
}

unsigned
cgwrap_is_usable (void)
{
  /* The library is linked at build time, thus always usable */
  return 1;
}

#endif /* !CGWRAP_ENABLE_DLOPEN */
