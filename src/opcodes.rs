pub enum OpCode {
    _0NNN(u16), // Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN. Not necessary for most ROMs. 
    _00E0, // Clears the screen.
    _00EE, // Returns from a subroutine.
    _1NNN, // Jumps to adress NNN.
    _2NNN, // Calls subroutine at NNN.
    _3XNN, // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
    _4XNN, // Skips the next instruction if VX does not equal NN. (Usually the next instruction is a jump to skip a code block)
    _5XY0, // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
    _6XNN, // Sets VX to NN
    _7XNN, // Adds NN to VX. (Carry flag is not changed)
    _8XY0, // Sets VX to the value of VY. 
    _8XY1, // Sets VX to VX or VY. (Bitwise OR operation)
    _8XY2, // Sets VX to VX and VY. (Bitwise AND operation)
    _8XY3, // Sets VX to VX xor VY. 
    _8XY4, // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not. 
    _8XY5, // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    _8XY6, // Stores the least significant bit of VX in VF and then shifts VX to the right by 1
    _8XY7, // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not. 
    _8XYE, // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    _9XY0, // Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block)
    _ANNN, // Sets I to the address NNN. 
    _BNNN, // Jumps to the address NNN plus V0. 
    _CXNN, // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN. 
    _DXYN, // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N+1 pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen
    _EX9E, // Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
    _EXA1, // Skips the next instruction if the key stored in VX is not pressed. (Usually the next instruction is a jump to skip a code block)
    _FX07, // Sets VX to the value of the delay timer. 
    _FX0A, // A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event); 
    _FX15, // Sets the delay timer to VX. 
    _FX18, // Sets the sound timer to VX. 
    _FX1E, // Adds VX to I. VF is not affected
    _FX29, // Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font. 
    _FX33, // Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
    _FX55, // Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
    _FX65, //  	Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
}
