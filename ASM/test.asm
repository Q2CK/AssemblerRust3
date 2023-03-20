#isa AnPUNano

#define bajo jajo

# test comment

.start
imm 0, 0
imm 1, 1

# test comment

.loop
add 0, 0, 1
add 1, 0, 1
jmp loop
