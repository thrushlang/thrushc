	.text
	.file	"BubbleSort.thrust"
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
	subq	$80, %rsp
	movq	%fs:40, %rax
	movq	%rax, -8(%rbp)
	movl	$64, -48(%rbp)
	movl	$34, -44(%rbp)
	movl	$25, -40(%rbp)
	movl	$12, -36(%rbp)
	movl	$22, -32(%rbp)
	movl	$11, -28(%rbp)
	movl	$54, -24(%rbp)
	movl	$9, -20(%rbp)
	movl	$10, -16(%rbp)
	movl	$90, -12(%rbp)
	movl	$.LstrygzUnP, %edi
	xorl	%eax, %eax
	movb	%al, -65(%rbp)
	callq	printf@PLT
	leaq	-48(%rbp), %rdi
	movq	%rdi, -64(%rbp)
	movl	$10, %esi
	movl	%esi, -52(%rbp)
	callq	.L__fn_YyWnrOMaPTxbRVwFdr_print_array
	movq	-64(%rbp), %rdi
	movl	-52(%rbp), %esi
	callq	.L__fn_lHTUZwPWQuDMvFphaaTSHfWGqAwzO_bubble_sort
	movb	-65(%rbp), %al
	movl	$.LstrEKMVmeh, %edi
	callq	printf@PLT
	movq	-64(%rbp), %rdi
	movl	-52(%rbp), %esi
	callq	.L__fn_YyWnrOMaPTxbRVwFdr_print_array
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_2
	xorl	%eax, %eax
	addq	$80, %rsp
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
	.type	.L__fn_YyWnrOMaPTxbRVwFdr_print_array,@function
.L__fn_YyWnrOMaPTxbRVwFdr_print_array:
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
	jae	.LBB1_5
	movq	-16(%rbp), %rcx
	movq	-24(%rbp), %rax
	movl	(%rax), %eax
	movslq	%eax, %rdx
	movl	(%rcx,%rdx,4), %esi
	movabsq	$.LstrifqsUWyLovBM, %rdi
	cltq
	movb	$0, %al
	callq	printf@PLT
	movq	-24(%rbp), %rax
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB1_2
.LBB1_5:
	movabsq	$.LstrVfbBsvIca, %rdi
	movb	$0, %al
	callq	printf@PLT
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	.L__fn_YyWnrOMaPTxbRVwFdr_print_array, .Lfunc_end1-.L__fn_YyWnrOMaPTxbRVwFdr_print_array
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_lHTUZwPWQuDMvFphaaTSHfWGqAwzO_bubble_sort,@function
.L__fn_lHTUZwPWQuDMvFphaaTSHfWGqAwzO_bubble_sort:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$32, %rsp
	movq	%rdi, -24(%rbp)
	movl	%esi, -12(%rbp)
	movl	$0, -4(%rbp)
	movl	$0, -8(%rbp)
.LBB2_1:
	movl	-12(%rbp), %ecx
	movl	-4(%rbp), %eax
	subl	$1, %ecx
	cmpl	%ecx, %eax
	jae	.LBB2_8
	movl	$0, -8(%rbp)
.LBB2_3:
	movl	-12(%rbp), %ecx
	movl	-8(%rbp), %eax
	subl	-4(%rbp), %ecx
	subl	$1, %ecx
	cmpl	%ecx, %eax
	jae	.LBB2_7
	movq	-24(%rbp), %rcx
	movslq	-8(%rbp), %rax
	movl	(%rcx,%rax,4), %eax
	movl	-8(%rbp), %edx
	addl	$1, %edx
	movslq	%edx, %rdx
	cmpl	(%rcx,%rdx,4), %eax
	jbe	.LBB2_6
	movq	-24(%rbp), %rdi
	movl	-8(%rbp), %esi
	movl	-8(%rbp), %edx
	addl	$1, %edx
	callq	.L__fn_PVVYBwtnvWeDIjJdbnJ_swap
.LBB2_6:
	movl	-8(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -8(%rbp)
	jmp	.LBB2_3
.LBB2_7:
	movl	-4(%rbp), %eax
	addl	$1, %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB2_1
.LBB2_8:
	addq	$32, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end2:
	.size	.L__fn_lHTUZwPWQuDMvFphaaTSHfWGqAwzO_bubble_sort, .Lfunc_end2-.L__fn_lHTUZwPWQuDMvFphaaTSHfWGqAwzO_bubble_sort
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_PVVYBwtnvWeDIjJdbnJ_swap,@function
.L__fn_PVVYBwtnvWeDIjJdbnJ_swap:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	movslq	%esi, %rax
	movl	(%rdi,%rax,4), %eax
	movslq	%esi, %rcx
	movl	%eax, -4(%rbp)
	movslq	%edx, %rax
	movl	(%rdi,%rax,4), %ecx
	movslq	%esi, %rax
	movslq	%edx, %rsi
	movl	%ecx, (%rdi,%rax,4)
	movl	-4(%rbp), %ecx
	movslq	%edx, %rax
	movl	%ecx, (%rdi,%rax,4)
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end3:
	.size	.L__fn_PVVYBwtnvWeDIjJdbnJ_swap, .Lfunc_end3-.L__fn_PVVYBwtnvWeDIjJdbnJ_swap
	.cfi_endproc

	.type	.LstrifqsUWyLovBM,@object
	.section	.rodata,"a",@progbits
.LstrifqsUWyLovBM:
	.asciz	"%d "
	.size	.LstrifqsUWyLovBM, 4

	.type	.LstrVfbBsvIca,@object
.LstrVfbBsvIca:
	.asciz	"\n"
	.size	.LstrVfbBsvIca, 2

	.type	.LstrygzUnP,@object
.LstrygzUnP:
	.asciz	"Original array: "
	.size	.LstrygzUnP, 17

	.type	.LstrEKMVmeh,@object
.LstrEKMVmeh:
	.asciz	"Sorted array:   "
	.size	.LstrEKMVmeh, 17

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
