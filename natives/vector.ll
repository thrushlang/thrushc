; ModuleID = 'output/vector.bc'
source_filename = "vector.th"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

declare void @free(ptr)

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.umax.i64(i64, i64) #0

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #1

declare noalias ptr @malloc(i64)

declare ptr @realloc(ptr, i64, ...)

define void @Vec.init(ptr %0, i64 %1, i64 %2, i8 %3, ...) {
  %5 = alloca i64, align 8
  %6 = alloca i64, align 8
  store i64 %1, ptr %5, align 8
  store i64 %2, ptr %6, align 8
  %7 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  store i64 0, ptr %7, align 8
  %8 = load i64, ptr %5, align 8
  %9 = call i64 @llvm.umax.i64(i64 2, i64 %8)
  %10 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 1
  store i64 %9, ptr %10, align 8
  %11 = load i64, ptr %6, align 8
  %12 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 2
  store i64 %11, ptr %12, align 8
  %13 = load i64, ptr %5, align 8
  %14 = load i64, ptr %6, align 8
  %15 = mul i64 %13, %14
  %16 = call ptr @malloc(i64 %15)
  %17 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  store ptr %16, ptr %17, align 8
  %18 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 4
  store i8 %3, ptr %18, align 1
  ret void
}

define void @Vec.destroy(ptr %0, ...) {
  %2 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %3 = load ptr, ptr %2, align 8
  tail call void @free(ptr %3)
  %4 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  store ptr null, ptr %4, align 8
  ret void
}

define private i1 @_Vec.should_grow(ptr %0, ...) {
  %2 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  %3 = load i64, ptr %2, align 8
  %4 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 1
  %5 = load i64, ptr %4, align 8
  %6 = icmp eq i64 %3, %5
  ret i1 %6
}

define void @Vec.realloc(ptr %0, i64 %1, i1 %2, ...) {
  %4 = icmp eq i1 %2, true
  br i1 %4, label %5, label %14

5:                                                ; preds = %3
  %6 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 2
  %7 = load i64, ptr %6, align 8
  %8 = add i64 %1, 2
  %9 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 1
  store i64 %8, ptr %9, align 8
  %10 = mul i64 %8, %7
  %11 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  store i64 0, ptr %11, align 8
  call void (ptr, ...) @Vec.destroy(ptr %0)
  %12 = call ptr @malloc(i64 %10)
  %13 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  store ptr %12, ptr %13, align 8
  ret void

14:                                               ; preds = %3
  %15 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %16 = load ptr, ptr %15, align 8
  %17 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 2
  %18 = load i64, ptr %17, align 8
  %19 = add i64 %1, 2
  %20 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 1
  store i64 %19, ptr %20, align 8
  %21 = mul i64 %19, %18
  %22 = call ptr (ptr, i64, ...) @realloc(ptr %16, i64 %21)
  %23 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  store ptr %22, ptr %23, align 8
  ret void
}

define private void @_Vec.adjust_capacity(ptr %0, ...) {
  %2 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  %3 = load i64, ptr %2, align 8
  %4 = mul i64 %3, 2
  %5 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  %6 = load i64, ptr %5, align 8
  %7 = call i64 @llvm.umax.i64(i64 %6, i64 %4)
  call void (ptr, i64, i1, ...) @Vec.realloc(ptr %0, i64 %7, i1 false)
  ret void
}

define i64 @Vec.size(ptr %0, ...) {
  %2 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  %3 = load i64, ptr %2, align 8
  ret i64 %3
}

define ptr @Vec.data(ptr %0, ...) {
  %2 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %3 = load ptr, ptr %2, align 8
  ret ptr %3
}

define void @Vec.push_i8(ptr %0, i8 %1, ...) {
  %3 = call i1 (ptr, ...) @_Vec.should_grow(ptr %0)
  %4 = icmp eq i1 %3, true
  br i1 %4, label %5, label %6

5:                                                ; preds = %2
  call void (ptr, ...) @_Vec.adjust_capacity(ptr %0)
  br label %6

6:                                                ; preds = %5, %2
  %7 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %8 = call ptr (ptr, ...) @Vec.data(ptr %0)
  %9 = getelementptr inbounds i8, ptr %8, i64 %7
  store i8 %1, ptr %9, align 1
  %10 = add i64 %7, 1
  %11 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  store i64 %10, ptr %11, align 8
  ret void
}

define void @Vec.push_i16(ptr %0, i16 %1, ...) {
  %3 = call i1 (ptr, ...) @_Vec.should_grow(ptr %0)
  %4 = icmp eq i1 %3, true
  br i1 %4, label %5, label %6

5:                                                ; preds = %2
  call void (ptr, ...) @_Vec.adjust_capacity(ptr %0)
  br label %6

6:                                                ; preds = %5, %2
  %7 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %8 = call ptr (ptr, ...) @Vec.data(ptr %0)
  %9 = getelementptr inbounds i8, ptr %8, i64 %7
  store i16 %1, ptr %9, align 2
  %10 = add i64 %7, 1
  %11 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  store i64 %10, ptr %11, align 8
  ret void
}

define void @Vec.push_i32(ptr %0, i32 %1, ...) {
  %3 = call i1 (ptr, ...) @_Vec.should_grow(ptr %0)
  %4 = icmp eq i1 %3, true
  br i1 %4, label %5, label %6

5:                                                ; preds = %2
  call void (ptr, ...) @_Vec.adjust_capacity(ptr %0)
  br label %6

6:                                                ; preds = %5, %2
  %7 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %8 = call ptr (ptr, ...) @Vec.data(ptr %0)
  %9 = getelementptr inbounds i8, ptr %8, i64 %7
  store i32 %1, ptr %9, align 4
  %10 = add i64 %7, 1
  %11 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  store i64 %10, ptr %11, align 8
  ret void
}

define void @Vec.push_i64(ptr %0, i64 %1, ...) {
  %3 = call i1 (ptr, ...) @_Vec.should_grow(ptr %0)
  %4 = icmp eq i1 %3, true
  br i1 %4, label %5, label %6

5:                                                ; preds = %2
  call void (ptr, ...) @_Vec.adjust_capacity(ptr %0)
  br label %6

6:                                                ; preds = %5, %2
  %7 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %8 = call ptr (ptr, ...) @Vec.data(ptr %0)
  %9 = getelementptr inbounds i8, ptr %8, i64 %7
  store i64 %1, ptr %9, align 8
  %10 = add i64 %7, 1
  %11 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 0
  store i64 %10, ptr %11, align 8
  ret void
}

define i8 @Vec.get_i8(ptr %0, i64 %1, ...) {
  %3 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %4 = load ptr, ptr %3, align 8
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = icmp ugt i64 %1, %5
  br i1 %6, label %7, label %8

7:                                                ; preds = %2
  ret i8 0

8:                                                ; preds = %2
  %9 = getelementptr inbounds i8, ptr %4, i64 %1
  %10 = load i8, ptr %9, align 1
  ret i8 %10
}

define i16 @Vec.get_i16(ptr %0, i64 %1, ...) {
  %3 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %4 = load ptr, ptr %3, align 8
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = icmp ugt i64 %1, %5
  br i1 %6, label %7, label %8

7:                                                ; preds = %2
  ret i16 0

8:                                                ; preds = %2
  %9 = getelementptr inbounds i16, ptr %4, i64 %1
  %10 = load i16, ptr %9, align 2
  ret i16 %10
}

define i32 @Vec.get_i32(ptr %0, i64 %1, ...) {
  %3 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %4 = load ptr, ptr %3, align 8
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = icmp ugt i64 %1, %5
  br i1 %6, label %7, label %8

7:                                                ; preds = %2
  ret i32 0

8:                                                ; preds = %2
  %9 = getelementptr inbounds i32, ptr %4, i64 %1
  %10 = load i32, ptr %9, align 4
  ret i32 %10
}

define i64 @Vec.get_i64(ptr %0, i64 %1, ...) {
  %3 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %4 = load ptr, ptr %3, align 8
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = icmp ugt i64 %1, %5
  br i1 %6, label %7, label %8

7:                                                ; preds = %2
  ret i64 0

8:                                                ; preds = %2
  %9 = getelementptr inbounds i64, ptr %4, i64 %1
  %10 = load i64, ptr %9, align 8
  ret i64 %10
}

define ptr @Vec.clone(ptr %0, ...) {
  %2 = tail call ptr @malloc(i32 ptrtoint (ptr getelementptr ({ i64, i64, i64, ptr, i8 }, ptr null, i32 1) to i32))
  call void @llvm.memcpy.p0.p0.i64(ptr %2, ptr %0, i64 ptrtoint (ptr getelementptr ({ i64, i64, i64, ptr, i8 }, ptr null, i32 1) to i64), i1 false)
  ret ptr %2
}

define void @Vec.set_i8(ptr %0, i64 %1, i8 %2, ...) {
  %4 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = sub i64 %5, 1
  %7 = icmp ugt i64 %1, %6
  br i1 %7, label %8, label %9

8:                                                ; preds = %3
  call void (ptr, i8, ...) @Vec.push_i8(ptr %0, i8 %2)
  ret void

9:                                                ; preds = %3
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds i8, ptr %10, i64 %1
  store i8 %2, ptr %11, align 1
  ret void
}

define void @Vec.set_i16(ptr %0, i64 %1, i16 %2, ...) {
  %4 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = sub i64 %5, 1
  %7 = icmp ugt i64 %1, %6
  br i1 %7, label %8, label %9

8:                                                ; preds = %3
  call void (ptr, i16, ...) @Vec.push_i16(ptr %0, i16 %2)
  ret void

9:                                                ; preds = %3
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds i16, ptr %10, i64 %1
  store i16 %2, ptr %11, align 2
  ret void
}

define void @Vec.set_i32(ptr %0, i64 %1, i32 %2, ...) {
  %4 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = sub i64 %5, 1
  %7 = icmp ugt i64 %1, %6
  br i1 %7, label %8, label %9

8:                                                ; preds = %3
  call void (ptr, i32, ...) @Vec.push_i32(ptr %0, i32 %2)
  ret void

9:                                                ; preds = %3
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds i32, ptr %10, i64 %1
  store i32 %2, ptr %11, align 4
  ret void
}

define void @Vec.set_i64(ptr %0, i64 %1, i64 %2, ...) {
  %4 = getelementptr inbounds { i64, i64, i64, ptr, i8 }, ptr %0, i32 0, i32 3
  %5 = call i64 (ptr, ...) @Vec.size(ptr %0)
  %6 = sub i64 %5, 1
  %7 = icmp ugt i64 %1, %6
  br i1 %7, label %8, label %9

8:                                                ; preds = %3
  call void (ptr, i64, ...) @Vec.push_i64(ptr %0, i64 %2)
  ret void

9:                                                ; preds = %3
  %10 = load ptr, ptr %4, align 8
  %11 = getelementptr inbounds i64, ptr %10, i64 %1
  store i64 %2, ptr %11, align 8
  ret void
}

attributes #0 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #1 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
