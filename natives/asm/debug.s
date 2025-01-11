	.text
	.file	"debug.th"
	.globl	panic                           # -- Begin function panic
	.p2align	4, 0x90
	.type	panic,@function
panic:                                  # @panic
	.cfi_startproc
# %bb.0:
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	(%rdi), %rdi
	movb	$0, %al
	callq	fprintf@PLT
.Lfunc_end0:
	.size	panic, .Lfunc_end0-panic
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
