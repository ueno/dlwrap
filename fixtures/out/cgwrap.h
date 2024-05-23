/*
 * Copying and distribution of this file, with or without modification,
 * are permitted in any medium without royalty provided the copyright
 * notice and this notice are preserved.  This file is offered as-is,
 * without any warranty.
 */

#ifndef CGWRAP_H_
#define CGWRAP_H_



#if defined(CGWRAP_ENABLE_DLOPEN) && CGWRAP_ENABLE_DLOPEN

#define FUNC(ret, name, args, cargs)		\
  ret cgwrap_func_##name args;
#define VOID_FUNC FUNC
#include "cgwrapfuncs.h"
#undef VOID_FUNC
#undef FUNC

#define CGWRAP_FUNC(name) cgwrap_func_##name

#else

#define CGWRAP_FUNC(name) name

#endif /* CGWRAP_ENABLE_DLOPEN */

/* Ensure SONAME to be loaded and all the necessary symbols are resolved.
 * Returns 0 on success; negative error code otherwise.
 *
 * Note that this function is NOT thread-safe; when calling it from
 * multi-threaded programs, protect it with a locking mechanism.
 */
int cgwrap_ensure_library (const char *soname);

/* Unload library and reset symbols.
 *
 * Note that this function is NOT thread-safe; when calling it from
 * multi-threaded programs, protect it with a locking mechanism.
 */
void cgwrap_unload_library (void);

#endif /* CGWRAP_H_ */
