	.text
	.file	"BinarySearch.thrust"
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
	subq	$64, %rsp
	movq	%fs:40, %rax
	movq	%rax, -8(%rbp)
	movl	$1, -48(%rbp)
	movl	$3, -44(%rbp)
	movl	$7, -40(%rbp)
	movl	$12, -36(%rbp)
	movl	$19, -32(%rbp)
	movl	$25, -28(%rbp)
	movl	$34, -24(%rbp)
	movl	$41, -20(%rbp)
	movl	$55, -16(%rbp)
	movl	$78, -12(%rbp)
	movq	8(%rsi), %rdi
	movb	$0, %al
	callq	atoi@PLT
	movl	%eax, -52(%rbp)
	movl	-52(%rbp), %esi
	leaq	-48(%rbp), %rdi
	callq	binary_search@PLT
	movl	%eax, -56(%rbp)
	cmpl	$0, -56(%rbp)
	jl	.LBB0_2
	movl	-52(%rbp), %esi
	movl	-56(%rbp), %edx
	movabsq	$.LstrmLLwinerK, %rdi
	movb	$0, %al
	callq	printf@PLT
	jmp	.LBB0_3
.LBB0_2:
	movl	-52(%rbp), %esi
	movabsq	$.LstrHFxNiMqNjVww, %rdi
	movb	$0, %al
	callq	printf@PLT
.LBB0_3:
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_5
	xorl	%eax, %eax
	addq	$64, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_5:
	.cfi_def_cfa %rbp, 16
	callq	__stack_chk_fail@PLT
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.globl	binary_search
	.p2align	4, 0x90
	.type	binary_search,@function
binary_search:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	movq	%rdi, -24(%rbp)
	movl	%esi, -16(%rbp)
	movl	$0, -4(%rbp)
	movl	$9, -8(%rbp)
	movl	$0, -12(%rbp)
.LBB1_1:
	movl	-4(%rbp), %eax
	cmpl	-8(%rbp), %eax
	jg	.LBB1_8
	movl	-4(%rbp), %eax
	addl	-8(%rbp), %eax
	movl	$2, %ecx
	cltd
	idivl	%ecx
	movl	-16(%rbp), %edx
	movl	%eax, %ecx
	movq	-24(%rbp), %rax
	movl	%ecx, -12(%rbp)
	movl	-12(%rbp), %ecx
	movslq	%ecx, %rcx
	cmpl	%edx, (%rax,%rcx,4)
	je	.LBB1_4
	movq	-24(%rbp), %rax
	movl	-16(%rbp), %edx
	movl	-12(%rbp), %ecx
	movslq	%ecx, %rcx
	cmpl	%edx, (%rax,%rcx,4)
	jl	.LBB1_5
	jmp	.LBB1_6
.LBB1_4:
	movl	-12(%rbp), %eax
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB1_5:
	.cfi_def_cfa %rbp, 16
	movl	-12(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB1_7
.LBB1_6:
	movl	-12(%rbp), %eax
	subl	$1, %eax
	movl	%eax, -8(%rbp)
.LBB1_7:
	jmp	.LBB1_1
.LBB1_8:
	movl	$4294967295, %eax
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	binary_search, .Lfunc_end1-binary_search
	.cfi_endproc

	.type	.LstrmLLwinerK,@object
	.section	.rodata,"a",@progbits
.LstrmLLwinerK:
	.asciz	"Found %d at index %d\n"
	.size	.LstrmLLwinerK, 22

	.type	.LstrHFxNiMqNjVww,@object
.LstrHFxNiMqNjVww:
	.asciz	"%d not found\n"
	.size	.LstrHFxNiMqNjVww, 14

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
