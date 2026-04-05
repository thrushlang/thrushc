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
	xorl	%eax, %eax
	addq	$16, %rax
	movl	%eax, -28(%rbp)
	movl	.Lglobal.constant.MFXeQmXAF_INET(%rip), %edi
	movl	.Lglobal.constant.rMtPubFzULiSOCK_STREAM(%rip), %esi
	xorl	%edx, %edx
	callq	socket@PLT
	movl	%eax, -32(%rbp)
	cmpl	$-1, -32(%rbp)
	je	.LBB0_2
	movl	.Lglobal.constant.MFXeQmXAF_INET(%rip), %eax
	movw	%ax, -24(%rbp)
	movw	$-28641, -22(%rbp)
	movl	.Lglobal.constant.aGYRkZCGINADDR_ANY(%rip), %eax
	movl	%eax, -20(%rbp)
	movl	-32(%rbp), %edi
	movl	-28(%rbp), %edx
	leaq	-24(%rbp), %rsi
	callq	bind@PLT
	cmpl	$0, %eax
	jl	.LBB0_5
	jmp	.LBB0_4
.LBB0_2:
	leaq	.LstrAUpaivVSWrFV(%rip), %rdi
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
	leaq	.LstrVuPTLuWEl(%rip), %rdi
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
	leaq	.LstrpcWFJfMCW(%rip), %rdi
	movb	$0, %al
	callq	printf@PLT
	leaq	.LstrftNoexeKaOYu(%rip), %rdi
	movb	$0, %al
	callq	printf@PLT
	jmp	.LBB0_10
.LBB0_8:
	leaq	.LstrsiuosKBTMocf(%rip), %rdi
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
	leaq	.LstrdYwFCnNSEpL(%rip), %rdi
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
	leaq	.LstrMUbnsuXjQ(%rip), %rdx
	movq	%rdx, -16(%rax)
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -48(%rbp)
	movq	%rax, %rsp
	xorl	%eax, %eax
	movl	%eax, %esi
	movq	%rsi, %rax
	addq	$217, %rax
	addq	$1, %rsi
	xorl	%edx, %edx
	divq	%rsi
	movq	%rax, %rdx
	movq	-48(%rbp), %rax
	movq	%rdx, (%rax)
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

	.type	.Lglobal.constant.aGYRkZCGINADDR_ANY,@object
	.section	.rodata.cst4,"aM",@progbits,4
	.p2align	2, 0x0
.Lglobal.constant.aGYRkZCGINADDR_ANY:
	.long	0
	.size	.Lglobal.constant.aGYRkZCGINADDR_ANY, 4

	.type	.Lglobal.constant.rMtPubFzULiSOCK_STREAM,@object
	.p2align	2, 0x0
.Lglobal.constant.rMtPubFzULiSOCK_STREAM:
	.long	1
	.size	.Lglobal.constant.rMtPubFzULiSOCK_STREAM, 4

	.type	.Lglobal.constant.MFXeQmXAF_INET,@object
	.p2align	2, 0x0
.Lglobal.constant.MFXeQmXAF_INET:
	.long	2
	.size	.Lglobal.constant.MFXeQmXAF_INET, 4

	.type	.LstrAUpaivVSWrFV,@object
	.section	.rodata,"a",@progbits
.LstrAUpaivVSWrFV:
	.asciz	"socket failed"
	.size	.LstrAUpaivVSWrFV, 14

	.type	.LstrVuPTLuWEl,@object
.LstrVuPTLuWEl:
	.asciz	"bind failed"
	.size	.LstrVuPTLuWEl, 12

	.type	.LstrsiuosKBTMocf,@object
.LstrsiuosKBTMocf:
	.asciz	"listen failed"
	.size	.LstrsiuosKBTMocf, 14

	.type	.LstrpcWFJfMCW,@object
.LstrpcWFJfMCW:
	.asciz	"HTTP Server running on http://127.0.0.1:8080\n"
	.size	.LstrpcWFJfMCW, 46

	.type	.LstrftNoexeKaOYu,@object
.LstrftNoexeKaOYu:
	.asciz	"Press Ctrl+C to stop...\n\n"
	.size	.LstrftNoexeKaOYu, 26

	.type	.LstrdYwFCnNSEpL,@object
.LstrdYwFCnNSEpL:
	.asciz	"accept failed"
	.size	.LstrdYwFCnNSEpL, 14

	.type	.LstrMUbnsuXjQ,@object
.LstrMUbnsuXjQ:
	.asciz	"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<html><head><title>Thrust Server</title></head><body><h1>\302\241Hola desde Thrust!</h1><p>Servidor funcionando correctamente.</p></body></html>"
	.size	.LstrMUbnsuXjQ, 217

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
