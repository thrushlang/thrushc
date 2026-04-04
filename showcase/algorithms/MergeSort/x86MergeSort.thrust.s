	.text
	.file	"MergeSort.thrust"
	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$112, %rsp
	movq	%fs:40, %rax
	movq	%rax, -8(%rbp)
	movl	$64, -48(%rbp)
	movl	$34, -44(%rbp)
	movl	$25, -40(%rbp)
	movl	$12, -36(%rbp)
	movl	$22, -32(%rbp)
	movl	$11, -28(%rbp)
	movl	$90, -24(%rbp)
	movl	$88, -20(%rbp)
	movl	$45, -16(%rbp)
	movl	$67, -12(%rbp)
	movl	$0, -88(%rbp)
	movl	$0, -84(%rbp)
	movl	$0, -80(%rbp)
	movl	$0, -76(%rbp)
	movl	$0, -72(%rbp)
	movl	$0, -68(%rbp)
	movl	$0, -64(%rbp)
	movl	$0, -60(%rbp)
	movl	$0, -56(%rbp)
	movl	$0, -52(%rbp)
	movl	$.LstrFLCRVWbF, %edi
	xorl	%eax, %eax
	movb	%al, -105(%rbp)
	callq	printf@PLT
	leaq	-48(%rbp), %rdi
	movq	%rdi, -104(%rbp)
	movl	$10, %esi
	movl	%esi, -92(%rbp)
	callq	.L__fn_RSUqeXVJEHTxRNOJqlUjsZIPSvb_print_array
	movq	-104(%rbp), %rdi
	movl	-92(%rbp), %edx
	leaq	-88(%rbp), %rsi
	callq	.L__fn_fgaCDHTjvmvfwgQSrvtNHFj_merge_sort
	movb	-105(%rbp), %al
	movl	$.LstrUhHTOjr, %edi
	callq	printf@PLT
	movq	-104(%rbp), %rdi
	movl	-92(%rbp), %esi
	callq	.L__fn_RSUqeXVJEHTxRNOJqlUjsZIPSvb_print_array
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_2
	xorl	%eax, %eax
	addq	$112, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_2:
	.cfi_def_cfa %rbp, 16
	callq	__stack_chk_fail@PLT
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_RSUqeXVJEHTxRNOJqlUjsZIPSvb_print_array,@function
.L__fn_RSUqeXVJEHTxRNOJqlUjsZIPSvb_print_array:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$32, %rsp
	movq	%rdi, -16(%rbp)
	movl	%esi, -4(%rbp)
	jmp	.LBB1_1
.LBB1_1:
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -24(%rbp)
	movq	%rax, %rsp
	movl	$0, (%rax)
.LBB1_2:
	movq	-24(%rbp), %rax
	movl	-4(%rbp), %ecx
	cmpl	%ecx, (%rax)
	jge	.LBB1_5
	movq	-16(%rbp), %rcx
	movq	-24(%rbp), %rax
	movl	(%rax), %eax
	movslq	%eax, %rdx
	movl	(%rcx,%rdx,4), %esi
	movabsq	$.LstrMTLxEqrTJmi, %rdi
	cltq
	movb	$0, %al
	callq	printf@PLT
	movq	-24(%rbp), %rax
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB1_2
.LBB1_5:
	movabsq	$.LstrFlSYuSMsqdED, %rdi
	movb	$0, %al
	callq	printf@PLT
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	.L__fn_RSUqeXVJEHTxRNOJqlUjsZIPSvb_print_array, .Lfunc_end1-.L__fn_RSUqeXVJEHTxRNOJqlUjsZIPSvb_print_array
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_fgaCDHTjvmvfwgQSrvtNHFj_merge_sort,@function
.L__fn_fgaCDHTjvmvfwgQSrvtNHFj_merge_sort:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$64, %rsp
	movq	%rdi, -32(%rbp)
	movq	%rsi, -24(%rbp)
	movl	%edx, -12(%rbp)
	movl	$1, -4(%rbp)
	movl	$0, -8(%rbp)
.LBB2_1:
	movl	-12(%rbp), %eax
	cmpl	%eax, -4(%rbp)
	jge	.LBB2_8
	movl	$0, -8(%rbp)
.LBB2_3:
	movl	-12(%rbp), %ecx
	movl	-8(%rbp), %eax
	subl	-4(%rbp), %ecx
	cmpl	%ecx, %eax
	jge	.LBB2_7
	movl	-12(%rbp), %ecx
	movq	%rsp, %rax
	movq	%rax, %rdx
	addq	$-16, %rdx
	movq	%rdx, -56(%rbp)
	movq	%rdx, %rsp
	movl	-8(%rbp), %edx
	movl	%edx, -16(%rax)
	movq	%rsp, %rax
	movq	%rax, %rdx
	addq	$-16, %rdx
	movq	%rdx, -48(%rbp)
	movq	%rdx, %rsp
	movl	-8(%rbp), %edi
	movl	-4(%rbp), %edx
	movl	%edx, %esi
	movl	%edi, %edx
	leal	-1(%rdx,%rsi), %edx
	movl	%edx, -16(%rax)
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -40(%rbp)
	movq	%rax, %rsp
	movl	-8(%rbp), %edx
	movl	-4(%rbp), %esi
	shll	%esi
	addl	%esi, %edx
	subl	$1, %edx
	movl	%edx, (%rax)
	cmpl	%ecx, (%rax)
	jl	.LBB2_6
	movq	-40(%rbp), %rax
	movl	-12(%rbp), %ecx
	subl	$1, %ecx
	movl	%ecx, (%rax)
.LBB2_6:
	movq	-24(%rbp), %rsi
	movq	-32(%rbp), %rdi
	movq	-40(%rbp), %rax
	movq	-48(%rbp), %rcx
	movq	-56(%rbp), %rdx
	movl	(%rdx), %edx
	movl	(%rcx), %ecx
	movl	(%rax), %r8d
	callq	.L__fn_FuJladRlzuPzPHPtorTdNSEErwGlI_merge
	movl	-8(%rbp), %eax
	movl	-4(%rbp), %ecx
	shll	%ecx
	addl	%ecx, %eax
	movl	%eax, -8(%rbp)
	jmp	.LBB2_3
.LBB2_7:
	movl	-4(%rbp), %eax
	shll	%eax
	movl	%eax, -4(%rbp)
	jmp	.LBB2_1
.LBB2_8:
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end2:
	.size	.L__fn_fgaCDHTjvmvfwgQSrvtNHFj_merge_sort, .Lfunc_end2-.L__fn_fgaCDHTjvmvfwgQSrvtNHFj_merge_sort
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_FuJladRlzuPzPHPtorTdNSEErwGlI_merge,@function
.L__fn_FuJladRlzuPzPHPtorTdNSEErwGlI_merge:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$48, %rsp
	movq	%rdi, -40(%rbp)
	movq	%rsi, -32(%rbp)
	movl	%edx, -24(%rbp)
	movl	%ecx, -20(%rbp)
	movl	%r8d, -16(%rbp)
	movl	%edx, -4(%rbp)
	addl	$1, %ecx
	movl	%ecx, -8(%rbp)
	movl	%edx, -12(%rbp)
.LBB3_1:
	movl	-20(%rbp), %eax
	cmpl	%eax, -4(%rbp)
	jle	.LBB3_4
	jmp	.LBB3_3
.LBB3_2:
	movq	-40(%rbp), %rcx
	movslq	-4(%rbp), %rax
	movl	(%rcx,%rax,4), %eax
	movl	-8(%rbp), %edx
	movslq	%edx, %rdx
	cmpl	(%rcx,%rdx,4), %eax
	jle	.LBB3_5
	jmp	.LBB3_6
.LBB3_3:
	jmp	.LBB3_8
.LBB3_4:
	movl	-16(%rbp), %eax
	cmpl	%eax, -8(%rbp)
	jle	.LBB3_2
	jmp	.LBB3_3
.LBB3_5:
	movq	-32(%rbp), %rax
	movq	-40(%rbp), %rdx
	movl	-12(%rbp), %ecx
	movl	-4(%rbp), %esi
	movslq	%esi, %rdi
	movl	(%rdx,%rdi,4), %edx
	movslq	%ecx, %rcx
	movslq	%esi, %rsi
	movl	%edx, (%rax,%rcx,4)
	movl	-4(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB3_7
.LBB3_6:
	movq	-32(%rbp), %rax
	movq	-40(%rbp), %rdx
	movl	-12(%rbp), %ecx
	movl	-8(%rbp), %esi
	movslq	%esi, %rdi
	movl	(%rdx,%rdi,4), %edx
	movslq	%ecx, %rcx
	movslq	%esi, %rsi
	movl	%edx, (%rax,%rcx,4)
	movl	-8(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -8(%rbp)
.LBB3_7:
	movl	-12(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -12(%rbp)
	jmp	.LBB3_1
.LBB3_8:
	movl	-20(%rbp), %eax
	cmpl	%eax, -4(%rbp)
	jle	.LBB3_10
	jmp	.LBB3_11
.LBB3_10:
	movq	-32(%rbp), %rax
	movq	-40(%rbp), %rdx
	movl	-12(%rbp), %ecx
	movl	-4(%rbp), %esi
	movslq	%esi, %rdi
	movl	(%rdx,%rdi,4), %edx
	movslq	%ecx, %rcx
	movslq	%esi, %rsi
	movl	%edx, (%rax,%rcx,4)
	movl	-4(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -4(%rbp)
	movl	%eax, -4(%rbp)
	movl	-12(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -12(%rbp)
	movl	%eax, -12(%rbp)
	jmp	.LBB3_8
.LBB3_11:
	movl	-16(%rbp), %eax
	cmpl	%eax, -8(%rbp)
	jle	.LBB3_13
	movl	-24(%rbp), %ecx
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -48(%rbp)
	movq	%rax, %rsp
	movl	%ecx, (%rax)
	jmp	.LBB3_14
.LBB3_13:
	movq	-32(%rbp), %rax
	movq	-40(%rbp), %rdx
	movl	-12(%rbp), %ecx
	movl	-8(%rbp), %esi
	movslq	%esi, %rdi
	movl	(%rdx,%rdi,4), %edx
	movslq	%ecx, %rcx
	movslq	%esi, %rsi
	movl	%edx, (%rax,%rcx,4)
	movl	-8(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -8(%rbp)
	movl	-12(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -12(%rbp)
	jmp	.LBB3_11
.LBB3_14:
	movq	-48(%rbp), %rax
	movl	-16(%rbp), %ecx
	cmpl	%ecx, (%rax)
	jg	.LBB3_16
	movq	-48(%rbp), %rax
	movq	-40(%rbp), %rcx
	movq	-32(%rbp), %rsi
	movl	(%rax), %edx
	movl	(%rax), %edi
	movslq	%edi, %r8
	movl	(%rsi,%r8,4), %esi
	movslq	%edx, %rdx
	movslq	%edi, %rdi
	movl	%esi, (%rcx,%rdx,4)
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB3_14
.LBB3_16:
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end3:
	.size	.L__fn_FuJladRlzuPzPHPtorTdNSEErwGlI_merge, .Lfunc_end3-.L__fn_FuJladRlzuPzPHPtorTdNSEErwGlI_merge
	.cfi_endproc

	.type	.LstrMTLxEqrTJmi,@object
	.section	.rodata,"a",@progbits
.LstrMTLxEqrTJmi:
	.asciz	"%d "
	.size	.LstrMTLxEqrTJmi, 4

	.type	.LstrFlSYuSMsqdED,@object
.LstrFlSYuSMsqdED:
	.asciz	"\n"
	.size	.LstrFlSYuSMsqdED, 2

	.type	.LstrFLCRVWbF,@object
.LstrFLCRVWbF:
	.asciz	"Original array: "
	.size	.LstrFLCRVWbF, 17

	.type	.LstrUhHTOjr,@object
.LstrUhHTOjr:
	.asciz	"Sorted array:   "
	.size	.LstrUhHTOjr, 17

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
