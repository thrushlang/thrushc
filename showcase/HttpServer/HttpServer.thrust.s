	.text
	.file	"HttpServer.thrust"
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
	subq	$48, %rsp
	movq	%fs:40, %rax
	movq	%rax, -8(%rbp)
	movl	$16, -28(%rbp)
	movl	.Lglobal.constant.eWGsGEENtJAF_INET, %edi
	movl	.Lglobal.constant.DyStUVcTSOCK_STREAM, %esi
	xorl	%edx, %edx
	callq	socket@PLT
	movl	%eax, -32(%rbp)
	cmpl	$-1, -32(%rbp)
	je	.LBB0_2
	movl	.Lglobal.constant.eWGsGEENtJAF_INET, %eax
	movw	%ax, -24(%rbp)
	movw	$-28641, -22(%rbp)
	movl	.Lglobal.constant.NqOgTINADDR_ANY, %eax
	movl	%eax, -20(%rbp)
	movl	-32(%rbp), %edi
	movl	-28(%rbp), %edx
	leaq	-24(%rbp), %rsi
	callq	bind@PLT
	cmpl	$0, %eax
	jl	.LBB0_5
	jmp	.LBB0_4
.LBB0_2:
	movl	$.LstrKsduhuhCWe, %edi
	xorl	%eax, %eax
	callq	perror@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_16
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_4:
	.cfi_def_cfa %rbp, 16
	movl	-32(%rbp), %edi
	movl	$3, %esi
	callq	listen@PLT
	cmpl	$0, %eax
	jl	.LBB0_8
	jmp	.LBB0_7
.LBB0_5:
	movl	$.LstrVfPvxcqAHpS, %edi
	xorl	%eax, %eax
	callq	perror@PLT
	movl	-32(%rbp), %edi
	callq	close@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_16
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_7:
	.cfi_def_cfa %rbp, 16
	movabsq	$.LstrvidXw, %rdi
	movb	$0, %al
	callq	printf@PLT
	movabsq	$.LstrYDzQScET, %rdi
	movb	$0, %al
	callq	printf@PLT
	jmp	.LBB0_10
.LBB0_8:
	movl	$.LstrcIudy, %edi
	xorl	%eax, %eax
	callq	perror@PLT
	movl	-32(%rbp), %edi
	callq	close@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_16
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_10:
	.cfi_def_cfa %rbp, 16
	movb	$1, %al
	testb	$1, %al
	jne	.LBB0_11
	jmp	.LBB0_14
.LBB0_11:
	movl	-32(%rbp), %edi
	leaq	-24(%rbp), %rsi
	leaq	-28(%rbp), %rdx
	callq	accept@PLT
	movl	%eax, -36(%rbp)
	cmpl	$0, -36(%rbp)
	jge	.LBB0_13
	movabsq	$.LstrADDUnaFh, %rdi
	movb	$0, %al
	callq	perror@PLT
	jmp	.LBB0_10
.LBB0_13:
	movq	%rsp, %rsi
	addq	$-1024, %rsi
	movq	%rsi, %rsp
	movl	-36(%rbp), %edi
	movl	$1024, %edx
	callq	read@PLT
	movq	%rsp, %rax
	movq	%rax, %rcx
	addq	$-16, %rcx
	movq	%rcx, %rsp
	movq	$.LstraohTtOT, -16(%rax)
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, %rsp
	movl	$180, (%rax)
	movl	-36(%rbp), %edi
	movq	(%rcx), %rsi
	movq	(%rax), %rax
	movl	%eax, %edx
	callq	write@PLT
	movl	-36(%rbp), %edi
	callq	close@PLT
	jmp	.LBB0_10
.LBB0_14:
	movl	-32(%rbp), %edi
	callq	close@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_16
	xorl	%eax, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_16:
	.cfi_def_cfa %rbp, 16
	callq	__stack_chk_fail@PLT
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.type	.Lglobal.constant.NqOgTINADDR_ANY,@object
	.section	.rodata.cst4,"aM",@progbits,4
	.p2align	2, 0x0
.Lglobal.constant.NqOgTINADDR_ANY:
	.long	0
	.size	.Lglobal.constant.NqOgTINADDR_ANY, 4

	.type	.Lglobal.constant.DyStUVcTSOCK_STREAM,@object
	.p2align	2, 0x0
.Lglobal.constant.DyStUVcTSOCK_STREAM:
	.long	1
	.size	.Lglobal.constant.DyStUVcTSOCK_STREAM, 4

	.type	.Lglobal.constant.eWGsGEENtJAF_INET,@object
	.p2align	2, 0x0
.Lglobal.constant.eWGsGEENtJAF_INET:
	.long	2
	.size	.Lglobal.constant.eWGsGEENtJAF_INET, 4

	.type	.LstrKsduhuhCWe,@object
	.section	.rodata,"a",@progbits
.LstrKsduhuhCWe:
	.asciz	"socket failed"
	.size	.LstrKsduhuhCWe, 14

	.type	.LstrVfPvxcqAHpS,@object
.LstrVfPvxcqAHpS:
	.asciz	"bind failed"
	.size	.LstrVfPvxcqAHpS, 12

	.type	.LstrcIudy,@object
.LstrcIudy:
	.asciz	"listen failed"
	.size	.LstrcIudy, 14

	.type	.LstrvidXw,@object
.LstrvidXw:
	.asciz	"HTTP Server running on http://127.0.0.1:8080\n"
	.size	.LstrvidXw, 46

	.type	.LstrYDzQScET,@object
.LstrYDzQScET:
	.asciz	"Press Ctrl+C to stop...\n\n"
	.size	.LstrYDzQScET, 26

	.type	.LstrADDUnaFh,@object
.LstrADDUnaFh:
	.asciz	"accept failed"
	.size	.LstrADDUnaFh, 14

	.type	.LstraohTtOT,@object
.LstraohTtOT:
	.asciz	"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<html><head><title>Thrust Http Server</title></head><body><h2>\302\241Hello from Thrust!</h2></body></html>"
	.size	.LstraohTtOT, 180

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
