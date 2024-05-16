/*
 * Copying and distribution of this file, with or without modification,
 * are permitted in any medium without royalty provided the copyright
 * notice and this notice are preserved.  This file is offered as-is,
 * without any warranty.
 */

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
  if (cgwrap_ensure_library (CGWRAP_SONAME) < 0)
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

/* Define redirection wrapper functions */
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

static int
ensure_symbol (const char *name, void **symp)
{
  if (!*symp)
    {
      void *sym = dlsym (cgwrap_dlhandle, name);
      if (!sym)
	return -errno;
      *symp = sym;
    }
}

int
cgwrap_ensure_library (const char *soname)
{
  int err;

  if (!cgwrap_dlhandle)
    {
      cgwrap_dlhandle = dlopen (soname, RTLD_LAZY | RTLD_LOCAL);
      if (!cgwrap_dlhandle)
	return -errno;
    }

#define ENSURE_SYMBOL(name)					\
  ensure_symbol(#name, (void **)&cgwrap_sym_##name)
#define FUNC(ret, name, args, cargs)	\
  err = ENSURE_SYMBOL(name);		\
  if (err < 0)				\
    return err;
#define VOID_FUNC FUNC
#include "cgwrapfuncs.h"
#undef VOID_FUNC
#undef FUNC
#undef ENSURE_SYMBOL
  return 0;
}

#else /* CGWRAP_ENABLE_DLOPEN */

int
cgwrap_ensure_library (const char *soname)
{
  (void) soname;
  return 0;
}

#endif /* !CGWRAP_ENABLE_DLOPEN */
