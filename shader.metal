#include <metal_stdlib>

using namespace metal;

typedef uint u32;

u32
m31_mul(u32 l, u32 r)
{
    const u32 P = 0x7fffffff;

    u32 lo = l * r;
    u32 hi = mulhi(l, r);

    u32 t = lo - hi * P;
    u32 u = t - P;

    return min(t, u);
}

kernel
void
sum(
    device       u32 *out [[buffer(0)]],
    device const u32 *a   [[buffer(1)]],
    device const u32 *b   [[buffer(2)]],
    u32 gid [[thread_position_in_grid]]
) {
    u32 l = a[gid], r = b[gid];
    for (int i = 0; i < 14; i++) {
        l = m31_mul(l, r);
    }
    out[gid] = l;
}

// volatile device atomic_uint *sum [[buffer(1)]],
// atomic_fetch_add_explicit(sum, data[gid], memory_order_relaxed);
