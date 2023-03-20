#isa AnPUNano


imm 0, 0
imm 1, 0
imm 2, 0
imm 3, 15
imm 4, 1

.in_loop
iml 0, 2
add 2, 2, 4
iml 1, 2
cmp 0, 1
brc 4 skip

ims 2, 0
sub 2, 2, 4
ims 2, 1
add 2, 2, 4

.skip
cmp 2, 3
brc 5, in_loop

sub 3, 3, 4
brc 0, in_loop
int 1, 0