	.text
	.file	"HeapSort.thrust"
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
	movl	$.LstrgIgzSjiQ, %edi
	xorl	%eax, %eax
	movb	%al, -65(%rbp)
	callq	printf@PLT
	leaq	-48(%rbp), %rdi
	movq	%rdi, -64(%rbp)
	movl	$10, %esi
	movl	%esi, -52(%rbp)
	callq	.L__fn_hoDyUoCwxyhWO_print_array
	movq	-64(%rbp), %rdi
	movl	-52(%rbp), %esi
	callq	.L__fn_WbnKwZpuTkLCRLmEnvpRXYOtJS_heap_sort
	movb	-65(%rbp), %al
	movl	$.LstrhsEFOcPOIu, %edi
	callq	printf@PLT
	movq	-64(%rbp), %rdi
	movl	-52(%rbp), %esi
	callq	.L__fn_hoDyUoCwxyhWO_print_array
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
	.type	.L__fn_hoDyUoCwxyhWO_print_array,@function
.L__fn_hoDyUoCwxyhWO_print_array:
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
	movabsq	$.LstrKTbVp, %rdi
	cltq
	movb	$0, %al
	callq	printf@PLT
	movq	-24(%rbp), %rax
	movl	(%rax), %ecx
	addl	$1, %ecx
	movl	%ecx, (%rax)
	jmp	.LBB1_2
.LBB1_5:
	movabsq	$.LstrpjoOWMOF, %rdi
	movb	$0, %al
	callq	printf@PLT
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end1:
	.size	.L__fn_hoDyUoCwxyhWO_print_array, .Lfunc_end1-.L__fn_hoDyUoCwxyhWO_print_array
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_WbnKwZpuTkLCRLmEnvpRXYOtJS_heap_sort,@function
.L__fn_WbnKwZpuTkLCRLmEnvpRXYOtJS_heap_sort:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$16, %rsp
	movl	%esi, %eax
	movq	%rdi, -16(%rbp)
	movl	%eax, -8(%rbp)
	movl	$2, %ecx
	cltd
	idivl	%ecx
	subl	$1, %eax
	movl	%eax, -4(%rbp)
.LBB2_1:
	cmpl	$0, -4(%rbp)
	jge	.LBB2_3
	movl	-8(%rbp), %eax
	subl	$1, %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB2_4
.LBB2_3:
	movl	-8(%rbp), %esi
	movq	-16(%rbp), %rdi
	movl	-4(%rbp), %edx
	callq	.L__fn_ndrkqDvXeD_heapify
	movl	-4(%rbp), %eax
	subl	$1, %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB2_1
.LBB2_4:
	cmpl	$0, -4(%rbp)
	jle	.LBB2_6
	movq	-16(%rbp), %rdi
	movl	-4(%rbp), %edx
	xorl	%esi, %esi
	callq	.L__fn_TWvocfcDBICNPh_swap
	movq	-16(%rbp), %rdi
	movl	-4(%rbp), %esi
	xorl	%edx, %edx
	callq	.L__fn_ndrkqDvXeD_heapify
	movl	-4(%rbp), %eax
	subl	$1, %eax
	movl	%eax, -4(%rbp)
	movl	%eax, -4(%rbp)
	jmp	.LBB2_4
.LBB2_6:
	addq	$16, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end2:
	.size	.L__fn_WbnKwZpuTkLCRLmEnvpRXYOtJS_heap_sort, .Lfunc_end2-.L__fn_WbnKwZpuTkLCRLmEnvpRXYOtJS_heap_sort
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_ndrkqDvXeD_heapify,@function
.L__fn_ndrkqDvXeD_heapify:
	.cfi_startproc
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	subq	$32, %rsp
	movl	%edx, %eax
	movq	%rdi, -32(%rbp)
	movl	%esi, -20(%rbp)
	movl	%eax, -16(%rbp)
	movl	%eax, -4(%rbp)
	movl	%eax, %ecx
	shll	%ecx
	addl	$1, %ecx
	movl	%ecx, -8(%rbp)
	shll	%eax
	addl	$2, %eax
	movl	%eax, -12(%rbp)
	cmpl	%esi, -8(%rbp)
	jl	.LBB3_2
.LBB3_1:
	movl	-20(%rbp), %eax
	cmpl	%eax, -12(%rbp)
	jl	.LBB3_5
	jmp	.LBB3_4
.LBB3_2:
	movq	-32(%rbp), %rcx
	movslq	-8(%rbp), %rax
	movl	(%rcx,%rax,4), %eax
	movl	-4(%rbp), %edx
	movslq	%edx, %rdx
	cmpl	(%rcx,%rdx,4), %eax
	jbe	.LBB3_1
	movl	-8(%rbp), %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB3_1
.LBB3_4:
	movl	-16(%rbp), %eax
	cmpl	%eax, -4(%rbp)
	jne	.LBB3_7
	jmp	.LBB3_8
.LBB3_5:
	movq	-32(%rbp), %rcx
	movslq	-12(%rbp), %rax
	movl	(%rcx,%rax,4), %eax
	movl	-4(%rbp), %edx
	movslq	%edx, %rdx
	cmpl	(%rcx,%rdx,4), %eax
	jbe	.LBB3_4
	movl	-12(%rbp), %eax
	movl	%eax, -4(%rbp)
	jmp	.LBB3_4
.LBB3_7:
	movq	-32(%rbp), %rdi
	movl	-16(%rbp), %esi
	movl	-4(%rbp), %edx
	callq	.L__fn_TWvocfcDBICNPh_swap
	movq	-32(%rbp), %rdi
	movl	-20(%rbp), %esi
	movl	-4(%rbp), %edx
	callq	.L__fn_ndrkqDvXeD_heapify
.LBB3_8:
	addq	$32, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.Lfunc_end3:
	.size	.L__fn_ndrkqDvXeD_heapify, .Lfunc_end3-.L__fn_ndrkqDvXeD_heapify
	.cfi_endproc

	.p2align	4, 0x90
	.type	.L__fn_TWvocfcDBICNPh_swap,@function
.L__fn_TWvocfcDBICNPh_swap:
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
	.size	.L__fn_TWvocfcDBICNPh_swap, .Lfunc_end4-.L__fn_TWvocfcDBICNPh_swap
	.cfi_endproc

	.type	.LstrKTbVp,@object
	.section	.rodata,"a",@progbits
.LstrKTbVp:
	.asciz	"%d "
	.size	.LstrKTbVp, 4

	.type	.LstrpjoOWMOF,@object
.LstrpjoOWMOF:
	.asciz	"\n"
	.size	.LstrpjoOWMOF, 2

	.type	.LstrgIgzSjiQ,@object
.LstrgIgzSjiQ:
	.asciz	"Original array: "
	.size	.LstrgIgzSjiQ, 17

	.type	.LstrhsEFOcPOIu,@object
.LstrhsEFOcPOIu:
	.asciz	"Sorted array:   "
	.size	.LstrhsEFOcPOIu, 17

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
