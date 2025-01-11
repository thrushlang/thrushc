	.text
	.file	"vector.th"
	.globl	Vec.init                        # -- Begin function Vec.init
	.p2align	4, 0x90
	.type	Vec.init,@function
Vec.init:                               # @Vec.init
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rdi, 8(%rsp)                   # 8-byte Spill
	movb	%cl, %al
	movb	%al, 23(%rsp)                   # 1-byte Spill
	movq	%rsi, 32(%rsp)
	movq	%rdx, 24(%rsp)
	movq	$0, (%rdi)
	movq	32(%rsp), %rcx
	movq	%rcx, %rax
	subq	$3, %rax
	movl	$2, %eax
	cmovaeq	%rcx, %rax
	movq	%rax, 8(%rdi)
	movq	24(%rsp), %rax
	movq	%rax, 16(%rdi)
	movq	32(%rsp), %rdi
	movq	24(%rsp), %rax
	imulq	%rax, %rdi
	callq	malloc@PLT
	movq	8(%rsp), %rdi                   # 8-byte Reload
	movq	%rax, %rcx
	movb	23(%rsp), %al                   # 1-byte Reload
	movq	%rcx, 24(%rdi)
	movb	%al, 32(%rdi)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	Vec.init, .Lfunc_end0-Vec.init
	.cfi_endproc
                                        # -- End function
	.globl	Vec.destroy                     # -- Begin function Vec.destroy
	.p2align	4, 0x90
	.type	Vec.destroy,@function
Vec.destroy:                            # @Vec.destroy
	.cfi_startproc
# %bb.0:
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rdi, (%rsp)                    # 8-byte Spill
	movq	24(%rdi), %rdi
	callq	free@PLT
	movq	(%rsp), %rdi                    # 8-byte Reload
	movq	$0, 24(%rdi)
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	Vec.destroy, .Lfunc_end1-Vec.destroy
	.cfi_endproc
                                        # -- End function
	.p2align	4, 0x90                         # -- Begin function _Vec.should_grow
	.type	.L_Vec.should_grow,@function
.L_Vec.should_grow:                     # @_Vec.should_grow
	.cfi_startproc
# %bb.0:
	movq	(%rdi), %rax
	movq	8(%rdi), %rcx
	subq	%rcx, %rax
	sete	%al
	retq
.Lfunc_end2:
	.size	.L_Vec.should_grow, .Lfunc_end2-.L_Vec.should_grow
	.cfi_endproc
                                        # -- End function
	.globl	Vec.realloc                     # -- Begin function Vec.realloc
	.p2align	4, 0x90
	.type	Vec.realloc,@function
Vec.realloc:                            # @Vec.realloc
	.cfi_startproc
# %bb.0:
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movq	%rsi, 8(%rsp)                   # 8-byte Spill
	movq	%rdi, 16(%rsp)                  # 8-byte Spill
	movb	%dl, %al
	testb	$1, %al
	je	.LBB3_2
	jmp	.LBB3_1
.LBB3_1:
	movq	16(%rsp), %rdi                  # 8-byte Reload
	movq	8(%rsp), %rax                   # 8-byte Reload
	movq	16(%rdi), %rcx
	addq	$2, %rax
	movq	%rax, 8(%rdi)
	imulq	%rcx, %rax
	movq	%rax, (%rsp)                    # 8-byte Spill
	movq	$0, (%rdi)
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	Vec.destroy@PLT
	movq	(%rsp), %rdi                    # 8-byte Reload
	callq	malloc@PLT
	movq	%rax, %rcx
	movq	16(%rsp), %rax                  # 8-byte Reload
	movq	%rcx, 24(%rax)
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB3_2:
	.cfi_def_cfa_offset 32
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movq	8(%rsp), %rsi                   # 8-byte Reload
	movq	16(%rcx), %rax
	movq	24(%rcx), %rdi
	addq	$2, %rsi
	movq	%rsi, 8(%rcx)
	imulq	%rax, %rsi
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	realloc@PLT
	movq	%rax, %rcx
	movq	16(%rsp), %rax                  # 8-byte Reload
	movq	%rcx, 24(%rax)
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end3:
	.size	Vec.realloc, .Lfunc_end3-Vec.realloc
	.cfi_endproc
                                        # -- End function
	.p2align	4, 0x90                         # -- Begin function _Vec.adjust_capacity
	.type	.L_Vec.adjust_capacity,@function
.L_Vec.adjust_capacity:                 # @_Vec.adjust_capacity
	.cfi_startproc
# %bb.0:
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	(%rdi), %rax
	movq	%rax, %rsi
	addq	%rsi, %rsi
	movq	%rax, %rcx
	subq	%rsi, %rcx
	cmovaq	%rax, %rsi
	xorl	%edx, %edx
	movb	%dl, %al
	callq	Vec.realloc@PLT
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end4:
	.size	.L_Vec.adjust_capacity, .Lfunc_end4-.L_Vec.adjust_capacity
	.cfi_endproc
                                        # -- End function
	.globl	Vec.size                        # -- Begin function Vec.size
	.p2align	4, 0x90
	.type	Vec.size,@function
Vec.size:                               # @Vec.size
	.cfi_startproc
# %bb.0:
	movq	(%rdi), %rax
	retq
.Lfunc_end5:
	.size	Vec.size, .Lfunc_end5-Vec.size
	.cfi_endproc
                                        # -- End function
	.globl	Vec.data                        # -- Begin function Vec.data
	.p2align	4, 0x90
	.type	Vec.data,@function
Vec.data:                               # @Vec.data
	.cfi_startproc
# %bb.0:
	movq	24(%rdi), %rax
	retq
.Lfunc_end6:
	.size	Vec.data, .Lfunc_end6-Vec.data
	.cfi_endproc
                                        # -- End function
	.globl	Vec.push_i8                     # -- Begin function Vec.push_i8
	.p2align	4, 0x90
	.type	Vec.push_i8,@function
Vec.push_i8:                            # @Vec.push_i8
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rdi, 24(%rsp)                  # 8-byte Spill
	movb	%sil, %al
	movb	%al, 39(%rsp)                   # 1-byte Spill
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	.L_Vec.should_grow
                                        # kill: def $cl killed $al
	testb	$1, %al
	je	.LBB7_2
	jmp	.LBB7_1
.LBB7_1:
	movq	24(%rsp), %rdi                  # 8-byte Reload
	movb	$0, %al
	callq	.L_Vec.adjust_capacity
.LBB7_2:
	movq	24(%rsp), %rdi                  # 8-byte Reload
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	movb	%al, 15(%rsp)                   # 1-byte Spill
	callq	Vec.size@PLT
	movq	24(%rsp), %rdi                  # 8-byte Reload
	movq	%rax, %rcx
	movb	15(%rsp), %al                   # 1-byte Reload
	movq	%rcx, 16(%rsp)                  # 8-byte Spill
	callq	Vec.data@PLT
	movb	39(%rsp), %sil                  # 1-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movq	%rax, %rdx
	movq	24(%rsp), %rax                  # 8-byte Reload
	movb	%sil, (%rdx,%rcx)
	incq	%rcx
	movq	%rcx, (%rax)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end7:
	.size	Vec.push_i8, .Lfunc_end7-Vec.push_i8
	.cfi_endproc
                                        # -- End function
	.globl	Vec.push_i16                    # -- Begin function Vec.push_i16
	.p2align	4, 0x90
	.type	Vec.push_i16,@function
Vec.push_i16:                           # @Vec.push_i16
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rdi, 24(%rsp)                  # 8-byte Spill
	movw	%si, %ax
	movw	%ax, 38(%rsp)                   # 2-byte Spill
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	.L_Vec.should_grow
                                        # kill: def $cl killed $al
	testb	$1, %al
	je	.LBB8_2
	jmp	.LBB8_1
.LBB8_1:
	movq	24(%rsp), %rdi                  # 8-byte Reload
	movb	$0, %al
	callq	.L_Vec.adjust_capacity
.LBB8_2:
	movq	24(%rsp), %rdi                  # 8-byte Reload
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	movb	%al, 15(%rsp)                   # 1-byte Spill
	callq	Vec.size@PLT
	movq	24(%rsp), %rdi                  # 8-byte Reload
	movq	%rax, %rcx
	movb	15(%rsp), %al                   # 1-byte Reload
	movq	%rcx, 16(%rsp)                  # 8-byte Spill
	callq	Vec.data@PLT
	movw	38(%rsp), %si                   # 2-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movq	%rax, %rdx
	movq	24(%rsp), %rax                  # 8-byte Reload
	movw	%si, (%rdx,%rcx)
	incq	%rcx
	movq	%rcx, (%rax)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end8:
	.size	Vec.push_i16, .Lfunc_end8-Vec.push_i16
	.cfi_endproc
                                        # -- End function
	.globl	Vec.push_i32                    # -- Begin function Vec.push_i32
	.p2align	4, 0x90
	.type	Vec.push_i32,@function
Vec.push_i32:                           # @Vec.push_i32
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movl	%esi, 28(%rsp)                  # 4-byte Spill
	movq	%rdi, 32(%rsp)                  # 8-byte Spill
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	.L_Vec.should_grow
                                        # kill: def $cl killed $al
	testb	$1, %al
	je	.LBB9_2
	jmp	.LBB9_1
.LBB9_1:
	movq	32(%rsp), %rdi                  # 8-byte Reload
	movb	$0, %al
	callq	.L_Vec.adjust_capacity
.LBB9_2:
	movq	32(%rsp), %rdi                  # 8-byte Reload
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	movb	%al, 15(%rsp)                   # 1-byte Spill
	callq	Vec.size@PLT
	movq	32(%rsp), %rdi                  # 8-byte Reload
	movq	%rax, %rcx
	movb	15(%rsp), %al                   # 1-byte Reload
	movq	%rcx, 16(%rsp)                  # 8-byte Spill
	callq	Vec.data@PLT
	movl	28(%rsp), %esi                  # 4-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movq	%rax, %rdx
	movq	32(%rsp), %rax                  # 8-byte Reload
	movl	%esi, (%rdx,%rcx)
	incq	%rcx
	movq	%rcx, (%rax)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end9:
	.size	Vec.push_i32, .Lfunc_end9-Vec.push_i32
	.cfi_endproc
                                        # -- End function
	.globl	Vec.push_i64                    # -- Begin function Vec.push_i64
	.p2align	4, 0x90
	.type	Vec.push_i64,@function
Vec.push_i64:                           # @Vec.push_i64
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rsi, 24(%rsp)                  # 8-byte Spill
	movq	%rdi, 32(%rsp)                  # 8-byte Spill
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	.L_Vec.should_grow
                                        # kill: def $cl killed $al
	testb	$1, %al
	je	.LBB10_2
	jmp	.LBB10_1
.LBB10_1:
	movq	32(%rsp), %rdi                  # 8-byte Reload
	movb	$0, %al
	callq	.L_Vec.adjust_capacity
.LBB10_2:
	movq	32(%rsp), %rdi                  # 8-byte Reload
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	movb	%al, 15(%rsp)                   # 1-byte Spill
	callq	Vec.size@PLT
	movq	32(%rsp), %rdi                  # 8-byte Reload
	movq	%rax, %rcx
	movb	15(%rsp), %al                   # 1-byte Reload
	movq	%rcx, 16(%rsp)                  # 8-byte Spill
	callq	Vec.data@PLT
	movq	24(%rsp), %rsi                  # 8-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movq	%rax, %rdx
	movq	32(%rsp), %rax                  # 8-byte Reload
	movq	%rsi, (%rdx,%rcx)
	incq	%rcx
	movq	%rcx, (%rax)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end10:
	.size	Vec.push_i64, .Lfunc_end10-Vec.push_i64
	.cfi_endproc
                                        # -- End function
	.globl	Vec.get_i8                      # -- Begin function Vec.get_i8
	.p2align	4, 0x90
	.type	Vec.get_i8,@function
Vec.get_i8:                             # @Vec.get_i8
	.cfi_startproc
# %bb.0:
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movq	%rsi, 16(%rsp)                  # 8-byte Spill
	movq	24(%rdi), %rax
	movq	%rax, 8(%rsp)                   # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	16(%rsp), %rsi                  # 8-byte Reload
	cmpq	%rax, %rsi
	jbe	.LBB11_2
# %bb.1:
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB11_2:
	.cfi_def_cfa_offset 32
	movq	8(%rsp), %rax                   # 8-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movb	(%rax,%rcx), %al
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end11:
	.size	Vec.get_i8, .Lfunc_end11-Vec.get_i8
	.cfi_endproc
                                        # -- End function
	.globl	Vec.get_i16                     # -- Begin function Vec.get_i16
	.p2align	4, 0x90
	.type	Vec.get_i16,@function
Vec.get_i16:                            # @Vec.get_i16
	.cfi_startproc
# %bb.0:
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movq	%rsi, 16(%rsp)                  # 8-byte Spill
	movq	24(%rdi), %rax
	movq	%rax, 8(%rsp)                   # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	16(%rsp), %rsi                  # 8-byte Reload
	cmpq	%rax, %rsi
	jbe	.LBB12_2
# %bb.1:
	xorl	%eax, %eax
                                        # kill: def $ax killed $ax killed $eax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB12_2:
	.cfi_def_cfa_offset 32
	movq	8(%rsp), %rax                   # 8-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movw	(%rax,%rcx,2), %ax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end12:
	.size	Vec.get_i16, .Lfunc_end12-Vec.get_i16
	.cfi_endproc
                                        # -- End function
	.globl	Vec.get_i32                     # -- Begin function Vec.get_i32
	.p2align	4, 0x90
	.type	Vec.get_i32,@function
Vec.get_i32:                            # @Vec.get_i32
	.cfi_startproc
# %bb.0:
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movq	%rsi, 16(%rsp)                  # 8-byte Spill
	movq	24(%rdi), %rax
	movq	%rax, 8(%rsp)                   # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	16(%rsp), %rsi                  # 8-byte Reload
	cmpq	%rax, %rsi
	jbe	.LBB13_2
# %bb.1:
	xorl	%eax, %eax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB13_2:
	.cfi_def_cfa_offset 32
	movq	8(%rsp), %rax                   # 8-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movl	(%rax,%rcx,4), %eax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end13:
	.size	Vec.get_i32, .Lfunc_end13-Vec.get_i32
	.cfi_endproc
                                        # -- End function
	.globl	Vec.get_i64                     # -- Begin function Vec.get_i64
	.p2align	4, 0x90
	.type	Vec.get_i64,@function
Vec.get_i64:                            # @Vec.get_i64
	.cfi_startproc
# %bb.0:
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movq	%rsi, 16(%rsp)                  # 8-byte Spill
	movq	24(%rdi), %rax
	movq	%rax, 8(%rsp)                   # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	16(%rsp), %rsi                  # 8-byte Reload
	cmpq	%rax, %rsi
	jbe	.LBB14_2
# %bb.1:
	xorl	%eax, %eax
                                        # kill: def $rax killed $eax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB14_2:
	.cfi_def_cfa_offset 32
	movq	8(%rsp), %rax                   # 8-byte Reload
	movq	16(%rsp), %rcx                  # 8-byte Reload
	movq	(%rax,%rcx,8), %rax
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end14:
	.size	Vec.get_i64, .Lfunc_end14-Vec.get_i64
	.cfi_endproc
                                        # -- End function
	.globl	Vec.clone                       # -- Begin function Vec.clone
	.p2align	4, 0x90
	.type	Vec.clone,@function
Vec.clone:                              # @Vec.clone
	.cfi_startproc
# %bb.0:
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rdi, (%rsp)                    # 8-byte Spill
	movl	$40, %edi
	callq	malloc@PLT
	movq	(%rsp), %rdi                    # 8-byte Reload
	movq	32(%rdi), %rcx
	movq	%rcx, 32(%rax)
	movq	24(%rdi), %rcx
	movq	%rcx, 24(%rax)
	movq	16(%rdi), %rcx
	movq	%rcx, 16(%rax)
	movq	(%rdi), %rcx
	movq	8(%rdi), %rdx
	movq	%rdx, 8(%rax)
	movq	%rcx, (%rax)
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end15:
	.size	Vec.clone, .Lfunc_end15-Vec.clone
	.cfi_endproc
                                        # -- End function
	.globl	Vec.set_i8                      # -- Begin function Vec.set_i8
	.p2align	4, 0x90
	.type	Vec.set_i8,@function
Vec.set_i8:                             # @Vec.set_i8
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rsi, 32(%rsp)                  # 8-byte Spill
	movq	%rdi, 8(%rsp)                   # 8-byte Spill
	movb	%dl, %al
	movb	%al, 23(%rsp)                   # 1-byte Spill
	movq	%rdi, %rax
	addq	$24, %rax
	movq	%rax, 24(%rsp)                  # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	32(%rsp), %rsi                  # 8-byte Reload
	subq	$1, %rax
	cmpq	%rax, %rsi
	jbe	.LBB16_2
# %bb.1:
	movq	8(%rsp), %rdi                   # 8-byte Reload
	movb	23(%rsp), %al                   # 1-byte Reload
	movzbl	%al, %esi
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	Vec.push_i8@PLT
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB16_2:
	.cfi_def_cfa_offset 48
	movq	32(%rsp), %rcx                  # 8-byte Reload
	movb	23(%rsp), %dl                   # 1-byte Reload
	movq	24(%rsp), %rax                  # 8-byte Reload
	movq	(%rax), %rax
	movb	%dl, (%rax,%rcx)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end16:
	.size	Vec.set_i8, .Lfunc_end16-Vec.set_i8
	.cfi_endproc
                                        # -- End function
	.globl	Vec.set_i16                     # -- Begin function Vec.set_i16
	.p2align	4, 0x90
	.type	Vec.set_i16,@function
Vec.set_i16:                            # @Vec.set_i16
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rsi, 32(%rsp)                  # 8-byte Spill
	movq	%rdi, 8(%rsp)                   # 8-byte Spill
	movw	%dx, %ax
	movw	%ax, 22(%rsp)                   # 2-byte Spill
	movq	%rdi, %rax
	addq	$24, %rax
	movq	%rax, 24(%rsp)                  # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	32(%rsp), %rsi                  # 8-byte Reload
	subq	$1, %rax
	cmpq	%rax, %rsi
	jbe	.LBB17_2
# %bb.1:
	movq	8(%rsp), %rdi                   # 8-byte Reload
	movw	22(%rsp), %ax                   # 2-byte Reload
                                        # implicit-def: $esi
	movw	%ax, %si
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	Vec.push_i16@PLT
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB17_2:
	.cfi_def_cfa_offset 48
	movq	32(%rsp), %rcx                  # 8-byte Reload
	movw	22(%rsp), %dx                   # 2-byte Reload
	movq	24(%rsp), %rax                  # 8-byte Reload
	movq	(%rax), %rax
	movw	%dx, (%rax,%rcx,2)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end17:
	.size	Vec.set_i16, .Lfunc_end17-Vec.set_i16
	.cfi_endproc
                                        # -- End function
	.globl	Vec.set_i32                     # -- Begin function Vec.set_i32
	.p2align	4, 0x90
	.type	Vec.set_i32,@function
Vec.set_i32:                            # @Vec.set_i32
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movl	%edx, 12(%rsp)                  # 4-byte Spill
	movq	%rsi, 32(%rsp)                  # 8-byte Spill
	movq	%rdi, 16(%rsp)                  # 8-byte Spill
	movq	%rdi, %rax
	addq	$24, %rax
	movq	%rax, 24(%rsp)                  # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	32(%rsp), %rsi                  # 8-byte Reload
	subq	$1, %rax
	cmpq	%rax, %rsi
	jbe	.LBB18_2
# %bb.1:
	movl	12(%rsp), %esi                  # 4-byte Reload
	movq	16(%rsp), %rdi                  # 8-byte Reload
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	Vec.push_i32@PLT
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB18_2:
	.cfi_def_cfa_offset 48
	movq	32(%rsp), %rcx                  # 8-byte Reload
	movl	12(%rsp), %edx                  # 4-byte Reload
	movq	24(%rsp), %rax                  # 8-byte Reload
	movq	(%rax), %rax
	movl	%edx, (%rax,%rcx,4)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end18:
	.size	Vec.set_i32, .Lfunc_end18-Vec.set_i32
	.cfi_endproc
                                        # -- End function
	.globl	Vec.set_i64                     # -- Begin function Vec.set_i64
	.p2align	4, 0x90
	.type	Vec.set_i64,@function
Vec.set_i64:                            # @Vec.set_i64
	.cfi_startproc
# %bb.0:
	subq	$40, %rsp
	.cfi_def_cfa_offset 48
	movq	%rdx, 8(%rsp)                   # 8-byte Spill
	movq	%rsi, 32(%rsp)                  # 8-byte Spill
	movq	%rdi, 16(%rsp)                  # 8-byte Spill
	movq	%rdi, %rax
	addq	$24, %rax
	movq	%rax, 24(%rsp)                  # 8-byte Spill
	movb	$0, %al
	callq	Vec.size@PLT
	movq	32(%rsp), %rsi                  # 8-byte Reload
	subq	$1, %rax
	cmpq	%rax, %rsi
	jbe	.LBB19_2
# %bb.1:
	movq	8(%rsp), %rsi                   # 8-byte Reload
	movq	16(%rsp), %rdi                  # 8-byte Reload
	xorl	%eax, %eax
                                        # kill: def $al killed $al killed $eax
	callq	Vec.push_i64@PLT
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB19_2:
	.cfi_def_cfa_offset 48
	movq	32(%rsp), %rcx                  # 8-byte Reload
	movq	8(%rsp), %rdx                   # 8-byte Reload
	movq	24(%rsp), %rax                  # 8-byte Reload
	movq	(%rax), %rax
	movq	%rdx, (%rax,%rcx,8)
	addq	$40, %rsp
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end19:
	.size	Vec.set_i64, .Lfunc_end19-Vec.set_i64
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
