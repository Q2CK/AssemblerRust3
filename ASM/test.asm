#isa AnPUNano

#define jeden 1

# test comment

.start
imm 0, 0
imm 1, 1

# test comment

.loop
add 0, 0, jeden
add 1, 0, 1
jmp loop
