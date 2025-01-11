; ModuleID = 'output/debug.bc'
source_filename = "debug.th"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

declare i32 @fprintf(ptr, ptr, ...)

define void @panic(ptr %0, ptr %1, ptr %2, ...) {
  %4 = load ptr, ptr %0, align 8
  %5 = call i32 (ptr, ptr, ...) @fprintf(ptr %4, ptr %1, ptr %2)
  unreachable
}
