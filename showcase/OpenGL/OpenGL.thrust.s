	.text
	.file	"OpenGL.thrust"
	.section	.rodata.cst4,"aM",@progbits,4
	.p2align	2, 0x0
.LCPI0_0:
	.long	0x3f800000
.LCPI0_1:
	.long	0x3f000000
.LCPI0_2:
	.long	0xbf000000
.LCPI0_3:
	.long	0x3dcccccd
.LCPI0_4:
	.long	0x3e4ccccd
	.text
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
	subq	$176, %rsp
	movq	%fs:40, %rax
	movq	%rax, -8(%rbp)
	leaq	.LstrsJQkRHHRQY(%rip), %rax
	movq	%rax, -88(%rbp)
	leaq	.LstrIEtBmkuRCC(%rip), %rax
	movq	%rax, -96(%rbp)
	vmovss	.LCPI0_2(%rip), %xmm0
	vmovss	%xmm0, -80(%rbp)
	vmovss	.LCPI0_2(%rip), %xmm0
	vmovss	%xmm0, -76(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -72(%rbp)
	vmovss	.LCPI0_0(%rip), %xmm0
	vmovss	%xmm0, -68(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -64(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -60(%rbp)
	vmovss	.LCPI0_1(%rip), %xmm0
	vmovss	%xmm0, -56(%rbp)
	vmovss	.LCPI0_2(%rip), %xmm0
	vmovss	%xmm0, -52(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -48(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -44(%rbp)
	vmovss	.LCPI0_0(%rip), %xmm0
	vmovss	%xmm0, -40(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -36(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -32(%rbp)
	vmovss	.LCPI0_1(%rip), %xmm0
	vmovss	%xmm0, -28(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -24(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -20(%rbp)
	vxorps	%xmm0, %xmm0, %xmm0
	vmovss	%xmm0, -16(%rbp)
	vmovss	.LCPI0_0(%rip), %xmm0
	vmovss	%xmm0, -12(%rbp)
	movl	$0, -100(%rbp)
	movl	$0, -104(%rbp)
	callq	glfwInit@PLT
	xorb	$-1, %al
	testb	$1, %al
	jne	.LBB0_2
	movl	$800, %edi
	movl	$600, %esi
	leaq	.LstrhvSbwHUI(%rip), %rdx
	xorl	%eax, %eax
	movl	%eax, %r8d
	movq	%r8, %rcx
	callq	glfwCreateWindow@PLT
	movq	%rax, -112(%rbp)
	cmpq	$0, -112(%rbp)
	je	.LBB0_5
	jmp	.LBB0_4
.LBB0_2:
	leaq	.LstrEPlTfg(%rip), %rdi
	xorl	%eax, %eax
	callq	perror@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_11
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_4:
	.cfi_def_cfa %rbp, 16
	movq	-112(%rbp), %rdi
	callq	glfwMakeContextCurrent@PLT
	movq	%rsp, %rax
	movq	%rax, -168(%rbp)
	addq	$-16, %rax
	movq	%rax, -136(%rbp)
	movq	%rax, %rsp
	movl	.Lglobal.constant.VAGSyaqoHQYyGL_VERTEX_SHADER(%rip), %edi
	callq	glCreateShader@PLT
	movl	%eax, %ecx
	movq	-168(%rbp), %rax
	movl	%ecx, -16(%rax)
	movl	-16(%rax), %edi
	xorl	%eax, %eax
	movl	%eax, %ecx
	movq	%rcx, -152(%rbp)
	movl	$1, %esi
	movl	%esi, -156(%rbp)
	leaq	-88(%rbp), %rdx
	callq	glShaderSource@PLT
	movq	-168(%rbp), %rax
	movl	-16(%rax), %edi
	callq	glCompileShader@PLT
	movq	%rsp, %rax
	movq	%rax, -144(%rbp)
	addq	$-16, %rax
	movq	%rax, -128(%rbp)
	movq	%rax, %rsp
	movl	.Lglobal.constant.AsuHPgaFsGL_FRAGMENT_SHADER(%rip), %edi
	callq	glCreateShader@PLT
	movl	-156(%rbp), %esi
	movq	-152(%rbp), %rcx
	movl	%eax, %edx
	movq	-144(%rbp), %rax
	movl	%edx, -16(%rax)
	movl	-16(%rax), %edi
	leaq	-96(%rbp), %rdx
	callq	glShaderSource@PLT
	movq	-144(%rbp), %rax
	movl	-16(%rax), %edi
	callq	glCompileShader@PLT
	movq	%rsp, %rax
	addq	$-16, %rax
	movq	%rax, -120(%rbp)
	movq	%rax, %rsp
	callq	glCreateProgram@PLT
	movq	-120(%rbp), %rcx
	movl	%eax, %edx
	movq	-136(%rbp), %rax
	movl	%edx, (%rcx)
	movl	(%rcx), %edi
	movl	(%rax), %esi
	callq	glAttachShader@PLT
	movq	-128(%rbp), %rax
	movq	-120(%rbp), %rcx
	movl	(%rcx), %edi
	movl	(%rax), %esi
	callq	glAttachShader@PLT
	movq	-120(%rbp), %rax
	movl	(%rax), %edi
	callq	glLinkProgram@PLT
	movq	-136(%rbp), %rax
	movl	(%rax), %edi
	callq	glDeleteShader@PLT
	movq	-128(%rbp), %rax
	movl	(%rax), %edi
	callq	glDeleteShader@PLT
	movl	$1, %edi
	leaq	-100(%rbp), %rsi
	callq	glGenVertexArrays@PLT
	movl	$1, %edi
	leaq	-104(%rbp), %rsi
	callq	glGenBuffers@PLT
	movl	-100(%rbp), %edi
	callq	glBindVertexArray@PLT
	movl	.Lglobal.constant.WbmwFKjHxigBGL_ARRAY_BUFFER(%rip), %edi
	movl	-104(%rbp), %esi
	callq	glBindBuffer@PLT
	movl	.Lglobal.constant.WbmwFKjHxigBGL_ARRAY_BUFFER(%rip), %edi
	leaq	-80(%rbp), %rdx
	movl	.Lglobal.constant.ViOlQCKGL_STATIC_DRAW(%rip), %ecx
	movl	$72, %esi
	callq	glBufferData@PLT
	movl	.Lglobal.constant.wHEPhLvpJGL_FLOAT(%rip), %edx
	movb	.Lglobal.constant.CVvkZdGL_FALSE(%rip), %al
	xorl	%ecx, %ecx
	movzbl	%al, %ecx
	xorl	%edi, %edi
	movl	%edi, %r9d
	movl	$3, %esi
	movl	$24, %r8d
	callq	glVertexAttribPointer@PLT
	xorl	%edi, %edi
	callq	glEnableVertexAttribArray@PLT
	movl	.Lglobal.constant.wHEPhLvpJGL_FLOAT(%rip), %edx
	movb	.Lglobal.constant.CVvkZdGL_FALSE(%rip), %al
	movzbl	%al, %ecx
	movl	$1, %edi
	movl	$3, %esi
	movl	$24, %r8d
	movl	$12, %r9d
	callq	glVertexAttribPointer@PLT
	movl	$1, %edi
	callq	glEnableVertexAttribArray@PLT
	movq	-120(%rbp), %rax
	movl	(%rax), %edi
	callq	glUseProgram@PLT
	leaq	.LstrqhFVszyWR(%rip), %rdi
	movb	$0, %al
	callq	printf@PLT
	leaq	.LstregJhsw(%rip), %rdi
	movb	$0, %al
	callq	printf@PLT
	jmp	.LBB0_7
.LBB0_5:
	leaq	.LstrQmchcEkpCh(%rip), %rdi
	xorl	%eax, %eax
	callq	perror@PLT
	callq	glfwTerminate@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_11
	movl	$1, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_7:
	.cfi_def_cfa %rbp, 16
	movq	-112(%rbp), %rdi
	callq	glfwWindowShouldClose@PLT
	xorb	$-1, %al
	testb	$1, %al
	jne	.LBB0_8
	jmp	.LBB0_9
.LBB0_8:
	vmovss	.LCPI0_3(%rip), %xmm1
	vmovss	.LCPI0_4(%rip), %xmm2
	vmovss	.LCPI0_0(%rip), %xmm3
	vmovaps	%xmm1, %xmm0
	callq	glClearColor@PLT
	movl	.Lglobal.constant.gTljSRGL_COLOR_BUFFER_BIT(%rip), %edi
	callq	glClear@PLT
	movl	-100(%rbp), %edi
	callq	glBindVertexArray@PLT
	movl	.Lglobal.constant.dKZMWeefnGL_TRIANGLES(%rip), %edi
	xorl	%esi, %esi
	movl	$3, %edx
	callq	glDrawArrays@PLT
	movq	-112(%rbp), %rdi
	callq	glfwSwapBuffers@PLT
	callq	glfwPollEvents@PLT
	jmp	.LBB0_7
.LBB0_9:
	callq	glfwTerminate@PLT
	movq	%fs:40, %rax
	movq	-8(%rbp), %rcx
	cmpq	%rcx, %rax
	jne	.LBB0_11
	xorl	%eax, %eax
	movq	%rbp, %rsp
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
.LBB0_11:
	.cfi_def_cfa %rbp, 16
	callq	__stack_chk_fail@PLT
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.type	.Lglobal.constant.AsuHPgaFsGL_FRAGMENT_SHADER,@object
	.section	.rodata.cst4,"aM",@progbits,4
	.p2align	2, 0x0
.Lglobal.constant.AsuHPgaFsGL_FRAGMENT_SHADER:
	.long	35632
	.size	.Lglobal.constant.AsuHPgaFsGL_FRAGMENT_SHADER, 4

	.type	.Lglobal.constant.VAGSyaqoHQYyGL_VERTEX_SHADER,@object
	.p2align	2, 0x0
.Lglobal.constant.VAGSyaqoHQYyGL_VERTEX_SHADER:
	.long	35633
	.size	.Lglobal.constant.VAGSyaqoHQYyGL_VERTEX_SHADER, 4

	.type	.Lglobal.constant.CVvkZdGL_FALSE,@object
	.section	.rodata,"a",@progbits
.Lglobal.constant.CVvkZdGL_FALSE:
	.byte	0
	.size	.Lglobal.constant.CVvkZdGL_FALSE, 1

	.type	.Lglobal.constant.dKZMWeefnGL_TRIANGLES,@object
	.section	.rodata.cst4,"aM",@progbits,4
	.p2align	2, 0x0
.Lglobal.constant.dKZMWeefnGL_TRIANGLES:
	.long	4
	.size	.Lglobal.constant.dKZMWeefnGL_TRIANGLES, 4

	.type	.Lglobal.constant.wHEPhLvpJGL_FLOAT,@object
	.p2align	2, 0x0
.Lglobal.constant.wHEPhLvpJGL_FLOAT:
	.long	5126
	.size	.Lglobal.constant.wHEPhLvpJGL_FLOAT, 4

	.type	.Lglobal.constant.ViOlQCKGL_STATIC_DRAW,@object
	.p2align	2, 0x0
.Lglobal.constant.ViOlQCKGL_STATIC_DRAW:
	.long	35044
	.size	.Lglobal.constant.ViOlQCKGL_STATIC_DRAW, 4

	.type	.Lglobal.constant.WbmwFKjHxigBGL_ARRAY_BUFFER,@object
	.p2align	2, 0x0
.Lglobal.constant.WbmwFKjHxigBGL_ARRAY_BUFFER:
	.long	34962
	.size	.Lglobal.constant.WbmwFKjHxigBGL_ARRAY_BUFFER, 4

	.type	.Lglobal.constant.gTljSRGL_COLOR_BUFFER_BIT,@object
	.p2align	2, 0x0
.Lglobal.constant.gTljSRGL_COLOR_BUFFER_BIT:
	.long	16384
	.size	.Lglobal.constant.gTljSRGL_COLOR_BUFFER_BIT, 4

	.type	.LstrsJQkRHHRQY,@object
	.section	.rodata,"a",@progbits
.LstrsJQkRHHRQY:
	.asciz	"#version 330 core\nlayout (location = 0) in vec3 aPos;\nlayout (location = 1) in vec3 aColor;\nout vec3 ourColor;\nvoid main() {\n    gl_Position = vec4(aPos, 1.0);\n    ourColor = aColor;\n}\000"
	.size	.LstrsJQkRHHRQY, 186

	.type	.LstrIEtBmkuRCC,@object
.LstrIEtBmkuRCC:
	.asciz	"#version 330 core\nout vec4 FragColor;\nin vec3 ourColor;\nvoid main() {\n    FragColor = vec4(ourColor, 1.0);\n}\000"
	.size	.LstrIEtBmkuRCC, 110

	.type	.LstrEPlTfg,@object
.LstrEPlTfg:
	.asciz	"glfwInit failed"
	.size	.LstrEPlTfg, 16

	.type	.LstrhvSbwHUI,@object
.LstrhvSbwHUI:
	.asciz	"Thrust OpenGL - Tri\303\241ngulo RGB"
	.size	.LstrhvSbwHUI, 31

	.type	.LstrQmchcEkpCh,@object
.LstrQmchcEkpCh:
	.asciz	"glfwCreateWindow failed"
	.size	.LstrQmchcEkpCh, 24

	.type	.LstrqhFVszyWR,@object
.LstrqhFVszyWR:
	.asciz	"Thrust OpenGL Triangle with Shaders running...\n"
	.size	.LstrqhFVszyWR, 48

	.type	.LstregJhsw,@object
.LstregJhsw:
	.asciz	"Press ESC or close window to exit.\n\n"
	.size	.LstregJhsw, 37

	.ident	"thrustc version 0.1.0"
	.section	".note.GNU-stack","",@progbits
