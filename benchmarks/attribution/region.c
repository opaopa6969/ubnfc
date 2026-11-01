/* Docker image の公式 client request header を使う。native 実行では no-op。 */
#include <valgrind/callgrind.h>
void ubnfc_profile_start(void) { CALLGRIND_START_INSTRUMENTATION; }
void ubnfc_profile_stop(void) { CALLGRIND_STOP_INSTRUMENTATION; }
