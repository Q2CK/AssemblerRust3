#isa AnPUNano

.start
imm 0, 255
imm 1, 255
.loop
add 0, 0, 1
add 1, 0, 1
jmp loop
