#include <stddef.h>
#include <stdint.h>
typedef struct {
    uint64_t lo;
    uint64_t hi;
} Wide;
typedef uint32_t (*Callback)(void *, const Wide *, const volatile unsigned char *);
_Static_assert(sizeof(Wide) == 16 && _Alignof(Wide) == 8, "Wide layout");
_Static_assert(offsetof(Wide, lo) == 0 && offsetof(Wide, hi) == 8, "Wide offsets");
uint32_t b38_probe(int64_t a, int64_t b, Wide *out, Callback callback, void *context)
{
    volatile unsigned char guard[16];
    guard[0] = 0;
    (void)guard[0];
    const uint64_t ua = (uint64_t)a;
    const uint64_t ub = (uint64_t)b;
    out->lo = ua - ub;
    out->hi = (a < 0 ? UINT64_MAX : 0) - (b < 0 ? UINT64_MAX : 0) - (ua < ub);
    const uint32_t status = callback(context, out, guard);
    if (status != 0)
        return status;
    return out->hi == 0 && out->lo <= 1000 ? 0 : 1;
}
