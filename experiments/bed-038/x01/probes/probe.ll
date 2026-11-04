source_filename = "b38-x01-probe"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"
%Wide = type { i64, i64 }
define i32 @b38_probe(i64 %a, i64 %b, ptr %out, ptr %callback, ptr %context) sspstrong {
entry:
  %guard = alloca [16 x i8], align 16
  store volatile i8 0, ptr %guard, align 16
  %touch = load volatile i8, ptr %guard, align 16
  %wa = sext i64 %a to i128
  %wb = sext i64 %b to i128
  %difference = sub i128 %wa, %wb
  %lo = trunc i128 %difference to i64
  %shift = lshr i128 %difference, 64
  %hi = trunc i128 %shift to i64
  %hi_ptr = getelementptr %Wide, ptr %out, i32 0, i32 1
  store i64 %lo, ptr %out, align 8
  store i64 %hi, ptr %hi_ptr, align 8
  %callback_status = call i32 %callback(ptr %context, ptr %out, ptr %guard)
  %callback_ok = icmp eq i32 %callback_status, 0
  %in_nominal = icmp ule i128 %difference, 1000
  %range_status = select i1 %in_nominal, i32 0, i32 1
  %status = select i1 %callback_ok, i32 %range_status, i32 %callback_status
  ret i32 %status
}
!llvm.module.flags = !{!0, !1}
!0 = !{i32 8, !"cf-protection-branch", i32 1}
!1 = !{i32 8, !"cf-protection-return", i32 1}
