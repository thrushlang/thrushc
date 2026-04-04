	.text
	.file	"QuickSort.thrust"
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
	leaq	.LstrdHCReQVKqwVY(%rip), %rdi
	xorl	%eax, %eax
	movl	%eax, -72(%rbp)
	movb	%al, -65(%rbp)
	callq	printf@PLT
	leaq	-48(%rbp), %rdi
	movq	%rdi, -64(%rbp)
	movl	$10, %esi
	movl	%esi, -52(%rbp)
	callq	.L__fn_ZiXOZGknLbrMvwlfvIRwDIDISvoeCJ_printArray
	movl	-72(%rbp), %esi
	movq	-64(%rbp), %rdi
	movl	$9, %edx
	callq	.L__fn_hhgmyEuNXi_quickSort
	movb	-65(%rbp), %al
	leaq	.LstrLRZOzexGmsS(%rip), %rdi
	callq	printf@PLT
	movq	-64(%rbp), %rdi
	movl	-52(%rbp), %esi
	callq	.L__fn_ZiXOZGknLbrMvwlfvIRwDIDISvoeCJ_printArray
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
	.type	.L__fn_ZiXOZGknLbrMvwlfvIRwDIDISvoeCJ_printArray,@function
.L__fn_ZiXOZGknLbrMvwlfvIRwDIDISvoeCJ_printArray:
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
	leaq	.LstrIFKUHJ(%rip), %rdi
	cltq
	movb	$0, %al
	callq	printf@PLT
	movq	-24(%rbp), %rax
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB1_2
.LBB1_5:
	leaq	.LstrJVbLVjmBKrx(%rip), %rdi
	movb	$0, %al
	callq	printf@PLT
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	.L__fn_ZiXOZGknLbrMvwlfvIRwDIDISvoeCJ_printArray, .Lfunc_end1-.L__fn_ZiXOZGknLbrMvwlfvIRwDIDISvoeCJ_printArray
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_hhgmyEuNXi_quickSort,@function
.L__fn_hhgmyEuNXi_quickSort:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$32, %rsp
	movq	%rdi, -16(%rbp)
	movl	%esi, -8(%rbp)
	movl	%edx, -4(%rbp)
	cmpl	%edx, %esi
	jge	.LBB2_2
	movl	-4(%rbp), %edx
	movq	-16(%rbp), %rdi
	movl	-8(%rbp), %esi
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -24(%rbp)
	movq	%rax, %rsp
	callq	.L__fn_sSMyDtCDEYnUPMJUWQROpOtoMokwi_partition
	movl	-8(%rbp), %esi
	movq	-16(%rbp), %rdi
	movl	%eax, %ecx
	movq	-24(%rbp), %rax
	movl	%ecx, (%rax)
	movl	(%rax), %edx
	subl	$1, %edx
	callq	.L__fn_hhgmyEuNXi_quickSort
	movq	-24(%rbp), %rax
	movq	-16(%rbp), %rdi
	movl	-4(%rbp), %edx
	movl	(%rax), %esi
	addl	$1, %esi
	callq	.L__fn_hhgmyEuNXi_quickSort
.LBB2_2:
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end2:
	.size	.L__fn_hhgmyEuNXi_quickSort, .Lfunc_end2-.L__fn_hhgmyEuNXi_quickSort
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_sSMyDtCDEYnUPMJUWQROpOtoMokwi_partition,@function
.L__fn_sSMyDtCDEYnUPMJUWQROpOtoMokwi_partition:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$32, %rsp
	movq	%rdi, -24(%rbp)
	movl	%esi, -16(%rbp)
	movl	%edx, -12(%rbp)
	movslq	%edx, %rax
	movl	(%rdi,%rax,4), %eax
	movslq	%edx, %rcx
	movl	%eax, -4(%rbp)
	subl	$1, %esi
	movl	%esi, -8(%rbp)
	movl	-16(%rbp), %ecx
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -32(%rbp)
	movq	%rax, %rsp
	movl	%ecx, (%rax)
.LBB3_2:
	movq	-32(%rbp), %rax
	movl	-12(%rbp), %ecx
	cmpl	%ecx, (%rax)
	jge	.LBB3_7
	movq	-24(%rbp), %rax
	movq	-32(%rbp), %rcx
	movslq	(%rcx), %rcx
	movl	(%rax,%rcx,4), %eax
	cmpl	-4(%rbp), %eax
	ja	.LBB3_5
	movq	-24(%rbp), %rdi
	movq	-32(%rbp), %rax
	movl	-8(%rbp), %ecx
	addl	$1, %ecx
	movl	%ecx, -8(%rbp)
	movl	-8(%rbp), %esi
	movl	(%rax), %edx
	callq	.L__fn_CRumSzkiVsdcmgsWQGJ_swap
.LBB3_5:
	jmp	.LBB3_6
.LBB3_6:
	movq	-32(%rbp), %rax
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB3_2
.LBB3_7:
	movl	-12(%rbp), %edx
	movq	-24(%rbp), %rdi
	movl	-8(%rbp), %esi
	addl	$1, %esi
	callq	.L__fn_CRumSzkiVsdcmgsWQGJ_swap
	movl	-8(%rbp), %eax
	addl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end3:
	.size	.L__fn_sSMyDtCDEYnUPMJUWQROpOtoMokwi_partition, .Lfunc_end3-.L__fn_sSMyDtCDEYnUPMJUWQROpOtoMokwi_partition
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_CRumSzkiVsdcmgsWQGJ_swap,@function
.L__fn_CRumSzkiVsdcmgsWQGJ_swap:
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
.Lfunc_end4:
	.size	.L__fn_CRumSzkiVsdcmgsWQGJ_swap, .Lfunc_end4-.L__fn_CRumSzkiVsdcmgsWQGJ_swap
	.cfi_endproc

	.type	.LstrIFKUHJ,@object
	.section	.rodata,"a",@progbits
.LstrIFKUHJ:
	.asciz	"%d "
	.size	.LstrIFKUHJ, 4

	.type	.LstrJVbLVjmBKrx,@object
.LstrJVbLVjmBKrx:
	.asciz	"\n"
	.size	.LstrJVbLVjmBKrx, 2

	.type	.LstrdHCReQVKqwVY,@object
.LstrdHCReQVKqwVY:
	.asciz	"Original array: "
	.size	.LstrdHCReQVKqwVY, 17

	.type	.LstrLRZOzexGmsS,@object
.LstrLRZOzexGmsS:
	.asciz	"Sorted array: "
	.size	.LstrLRZOzexGmsS, 15

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
