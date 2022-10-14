    .intel_syntax noprefix
    .global  asm_i32_from_str16_sse

    .section .data
    .align 64
pcmpistri_cntl: .byte 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39
    .space 6, 0x00
ascii_adjust: .space 16, 0x0F
mul1: .byte 1, 10, 1, 10, 1, 10, 1, 10
      .byte 1, 10, 1, 10, 1, 10, 1, 10
mul2: .2byte 1, 100, 1, 100, 1, 100, 1, 100
mul3: .2byte 1, 10000, 1, 10000, 1, 10000, 1, 10000
reverse: .byte 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08
         .byte 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00
zeros: .space 16, 0x30

asm_n32_from_str16_sse_switch:
    .8byte asm_n32_from_str16_sse_switch_end
    .8byte asm_n32_from_str16_sse_case_1
    .8byte asm_n32_from_str16_sse_case_2
    .8byte asm_n32_from_str16_sse_case_3
    .8byte asm_n32_from_str16_sse_case_4
    .8byte asm_n32_from_str16_sse_case_5
    .8byte asm_n32_from_str16_sse_case_6
    .8byte asm_n32_from_str16_sse_case_7
    .8byte asm_n32_from_str16_sse_case_8
    .8byte asm_n32_from_str16_sse_case_9
    .8byte asm_n32_from_str16_sse_case_10
    .8byte asm_n32_from_str16_sse_case_11
    .8byte asm_n32_from_str16_sse_case_12
    .8byte asm_n32_from_str16_sse_case_13
    .8byte asm_n32_from_str16_sse_case_14
    .8byte asm_n32_from_str16_sse_case_15
    .8byte asm_n32_from_str16_sse_switch_end


        .section .text
# \param
# rdi: pointer to buffer
# rsi: length of buffer
# \return
# rax: return value
# rdx: length of decoded number
asm_n32_from_str16_sse:
    push       rcx
    sub        rsp,      8

    vzeroupper
    movdqa     xmm7,     xmmword ptr [zeros]

1:
    movdqu     xmm6,     xmmword ptr [rdi]
    pcmpistri  xmm7,     xmm6,     0b00010000
#    pcmpeqb    xmm6,     xmm7
#    pmovmskb   ecx,      xmm6
#    not        ecx
#    tzcnt      ecx,      ecx
    cmp        rcx,      rsi
    cmovg      rcx,      rsi
    add        rdi,      rcx
    sub        rsi,      rcx
    xor        ecx,      0x10
    jz         1b

    movdqa     xmm7,     xmmword ptr [pcmpistri_cntl]
    movdqa     xmm6,     xmmword ptr [ascii_adjust]
    movdqa     xmm2,     xmmword ptr [reverse]
    pxor       xmm1,     xmm1

    movdqu     xmm0,     xmmword ptr [rdi]
    pcmpistri  xmm7,     xmm0,     0b00010000

    movdqa     xmm5,     xmmword ptr [mul1]
    movdqa     xmm4,     xmmword ptr [mul2]
    movdqa     xmm3,     xmmword ptr [mul3]

    pand       xmm0,     xmm6
    pshufb     xmm0,     xmm2

    cmp        rcx,      rsi
    cmovg      rcx,      rsi
    sub        rsi,      rcx

    jmp        qword ptr [asm_n32_from_str16_sse_switch + ecx * 8]
asm_n32_from_str16_sse_switch_end:

    pmaddubsw xmm1,      xmm5
    pmaddwd   xmm1,      xmm4
    packusdw  xmm1,      xmm1
    pmaddwd   xmm1,      xmm3

    pextrd    eax,       xmm1,     0
    pextrd    edx,       xmm1,     1

    imul      rdx,       100000000
    add       rax,       rdx

    mov       rcx,       0xFFFFFFFF << 1
    cmp       rsi,       0
    cmovl     rax,       rcx

0:
    add       rsp,       8
    pop       rcx
    ret

.macro asm_n32_from_str16_sse_case_n number
asm_n32_from_str16_sse_case_\number\():
    palignr   xmm1,      xmm0,     (16 - \number\())
    jmp       asm_n32_from_str16_sse_switch_end
.endm

asm_n32_from_str16_sse_case_n 1
asm_n32_from_str16_sse_case_n 2
asm_n32_from_str16_sse_case_n 3
asm_n32_from_str16_sse_case_n 4
asm_n32_from_str16_sse_case_n 5
asm_n32_from_str16_sse_case_n 6
asm_n32_from_str16_sse_case_n 7
asm_n32_from_str16_sse_case_n 8
asm_n32_from_str16_sse_case_n 9
asm_n32_from_str16_sse_case_n 10
asm_n32_from_str16_sse_case_n 11
asm_n32_from_str16_sse_case_n 12
asm_n32_from_str16_sse_case_n 13
asm_n32_from_str16_sse_case_n 14
asm_n32_from_str16_sse_case_n 15


# \param
# rdi: pointer to buffer
# rsi: length of buffer
# \return
# rax: return value
# rdx: length of decoded number
asm_i32_from_str16_sse:
    push    rbx
    push    rcx

    xor     ebx,     ebx
    xor     ecx,     ecx
    cmp     byte ptr [rdi], '-'
    setz    bl

    add     rdi,     rbx
    sub     rsi,     rbx
    sub     ecx,     ebx

    call    asm_n32_from_str16_sse

    mov     edx,     -1

    cmp     rax,     rdx
    cmova   eax,     edx

    xor     eax,     ecx
    add     eax,     ebx

    pop     rcx
    pop     rbx
    ret

