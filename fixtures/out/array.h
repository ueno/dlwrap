/*
 * Copying and distribution of this file, with or without modification,
 * are permitted in any medium without royalty provided the copyright
 * notice and this notice are preserved.  This file is offered as-is,
 * without any warranty.
 */

#ifndef ARRAY_H_
#define ARRAY_H_



#if defined(ARRAY_ENABLE_DLOPEN) && ARRAY_ENABLE_DLOPEN

#define FUNC(ret, name, args, cargs)		\
  ret array_func_##name args;
#define VOID_FUNC FUNC
#include "arrayfuncs.h"
#undef VOID_FUNC
#undef FUNC

#define ARRAY_FUNC(name) array_func_##name

#else

#define ARRAY_FUNC(name) name

#endif /* ARRAY_ENABLE_DLOPEN */

/* Ensure SONAME to be loaded with dlopen FLAGS, and all the necessary
 * symbols are resolved.
 *
 * Returns 0 on success; negative error code otherwise.
 *
 * Note that this function is NOT thread-safe; when calling it from
 * multi-threaded programs, protect it with a locking mechanism.
 */
int array_ensure_library (const char *soname, int flags);

/* Unload library and reset symbols.
 *
 * Note that this function is NOT thread-safe; when calling it from
 * multi-threaded programs, protect it with a locking mechanism.
 */
void array_unload_library (void);

/* Return 1 if the library is loaded and usable.
 *
 * Note that this function is NOT thread-safe; when calling it from
 * multi-threaded programs, protect it with a locking mechanism.
 */
unsigned array_is_usable (void);

#endif /* ARRAY_H_ */
