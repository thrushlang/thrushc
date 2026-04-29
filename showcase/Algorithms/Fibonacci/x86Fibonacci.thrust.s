	.text
	.file	"Fibonacci.thrust"
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
	subq	$32, %rsp
	movq	%rsi, -8(%rbp)
	cmpl	$2, %edi
	jl	.LBB0_2
	movq	-8(%rbp), %rax
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, -16(%rbp)
	movq	%rcx, %rsp
	movq	8(%rax), %rdi
	movb	$0, %al
	callq	atoi@PLT
	movl	%eax, %ecx
	movq	-16(%rbp), %rax
	movl	%ecx, (%rax)
	cmpl	$0, (%rax)
	jl	.LBB0_3
	jmp	.LBB0_4
.LBB0_2:
	movabsq	$.LstrvzeKjgBoZJK, %rdi
	movb	$0, %al
	callq	printf@PLT
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_3:
	.cfi_def_cfa %rbp, 16
	movabsq	$.LstrvLFDGBd, %rdi
	movb	$0, %al
	callq	printf@PLT
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_4:
	.cfi_def_cfa %rbp, 16
	movq	-16(%rbp), %rax
	movq	%rsp, %rcx
	addq	$-16, %rcx
	movq	%rcx, -24(%rbp)
	movq	%rcx, %rsp
	movl	(%rax), %edi
	callq	fibonacci@PLT
	movq	-16(%rbp), %rcx
	movl	%eax, %edx
	movq	-24(%rbp), %rax
	movl	%edx, (%rax)
	movl	(%rcx), %esi
	movl	(%rax), %edx
	movabsq	$.LstrfihTr, %rdi
	movb	$0, %al
	callq	printf@PLT
	xorl	%eax, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.globl	fibonacci
	.p2align	4, 0x90
	.type	fibonacci,@function
fibonacci:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$48, %rsp
	movl	%edi, -4(%rbp)
	cmpl	$0, %edi
	jle	.LBB1_2
	movl	-4(%rbp), %eax
	cmpl	$1, %eax
	je	.LBB1_4
	jmp	.LBB1_3
.LBB1_2:
	xorl	%eax, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB1_3:
	.cfi_def_cfa %rbp, 16
	movq	%rsp, %rax
	movq	%rax, %rcx
	addq	$-16, %rcx
	movq	%rcx, -40(%rbp)
	movq	%rcx, %rsp
	movl	$0, -16(%rax)
	movq	%rsp, %rax
	movq	%rax, %rcx
	addq	$-16, %rcx
	movq	%rcx, -32(%rbp)
	movq	%rcx, %rsp
	movl	$1, -16(%rax)
	movq	%rsp, %rax
	movq	%rax, %rcx
	addq	$-16, %rcx
	movq	%rcx, -24(%rbp)
	movq	%rcx, %rsp
	movl	$2, -16(%rax)
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -16(%rbp)
	movq	%rax, %rsp
	movl	$0, (%rax)
	jmp	.LBB1_5
.LBB1_4:
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB1_5:
	.cfi_def_cfa %rbp, 16
	movq	-24(%rbp), %rax
	movl	-4(%rbp), %ecx
	cmpl	%ecx, (%rax)
	jg	.LBB1_7
	movq	-24(%rbp), %rax
	movq	-32(%rbp), %rcx
	movq	-16(%rbp), %rdx
	movq	-40(%rbp), %rsi
	movl	(%rsi), %edi
	addl	(%rcx), %edi
	movl	%edi, (%rdx)
	movl	(%rcx), %edi
	movl	%edi, (%rsi)
	movl	(%rdx), %edx
	movl	%edx, (%rcx)
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB1_5
.LBB1_7:
	movq	-16(%rbp), %rax
	movl	(%rax), %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	fibonacci, .Lfunc_end1-fibonacci
	.cfi_endproc

	.type	.LstrvzeKjgBoZJK,@object
	.section	.rodata,"a",@progbits
.LstrvzeKjgBoZJK:
	.asciz	"Usage: ./fibonacci <n>\n"
	.size	.LstrvzeKjgBoZJK, 24

	.type	.LstrvLFDGBd,@object
.LstrvLFDGBd:
	.asciz	"Please enter a non-negative number\n"
	.size	.LstrvLFDGBd, 36

	.type	.LstrfihTr,@object
.LstrfihTr:
	.asciz	"fib(%d) = %d\n"
	.size	.LstrfihTr, 14

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
