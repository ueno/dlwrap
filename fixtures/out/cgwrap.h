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
 */
int cgwrap_ensure_library (const char *soname);

#endif /* CGWRAP_H_ */
